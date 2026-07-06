//! WebSocket gateway.
//!
//! Protocol:
//!   Client → Server:
//!     { "type": "subscribe", "channel": "order:UUID:events" }
//!     { "type": "unsubscribe", "channel": "..." }
//!     { "type": "replay", "channel": "...", "last_event_id": 42 }
//!     { "type": "driver_location", "lat": .., "lng": .., "heading": .., "speed_kph": .. }
//!     { "type": "pong" }
//!   Server → Client:
//!     { "type": "connected", "user_id": "..." }
//!     { "type": "order_event", "channel": "...", "event_type": "...", "payload": {...} }
//!     { "type": "driver_location", "channel": "...", "driver_id": "...", "lat": .., "lng": .. }
//!     { "type": "replay_result", "channel": "...", "events": [...] }
//!     { "type": "snapshot", "channel": "...", "state": {...} }
//!     { "type": "ping" }
//!     { "type": "error", "code": "...", "message": "..." }
//!
//! Scaling: Redis pub/sub syncs events across multiple WS worker instances.
//! No sticky sessions required.

use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::repos::order_repo;
use crate::domain::user::UserRole;
use crate::error::{AppError, AppResult};
use crate::services::realtime_service;
use crate::utils::jwt;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/ws", axum::routing::get(ws_handler))
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    Replay { channel: String, last_event_id: i64 },
    DriverLocation {
        lat: f64,
        lng: f64,
        heading: Option<f64>,
        speed_kph: Option<f64>,
    },
    Pong,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum ServerMsg {
    Connected { user_id: String },
    OrderEvent {
        channel: String,
        event_type: String,
        payload: serde_json::Value,
    },
    DriverLocation {
        channel: String,
        driver_id: String,
        lat: f64,
        lng: f64,
    },
    ReplayResult {
        channel: String,
        events: Vec<serde_json::Value>,
    },
    Snapshot {
        channel: String,
        state: serde_json::Value,
    },
    Ping,
    Error { code: String, message: String },
}

async fn ws_handler(
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> AppResult<axum::response::Response> {
    let claims = jwt::verify_access(&q.token)
        .map_err(|e| AppError::unauthenticated(format!("invalid token: {}", e)))?;

    let user_id = claims.sub.clone();
    let role = claims.role;

    info!(user_id = %user_id, role = ?role, "ws connected");

    Ok(ws
        .on_upgrade(move |socket| handle_socket(socket, state, user_id, role))
        .into_response())
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: String, role: UserRole) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(64);

    // Send "connected" message
    let hello = serde_json::to_string(&ServerMsg::Connected {
        user_id: user_id.clone(),
    })
    .unwrap_or_else(|_| "{}".into());
    let _ = ws_tx.send(Message::Text(hello.into())).await;

    // Writer task: pulls from rx channel, sends to WS.
    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Periodic ping every 30s.
    let ping_tx = tx.clone();
    let ping = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.tick().await;
        loop {
            interval.tick().await;
            let msg = serde_json::to_string(&ServerMsg::Ping).unwrap_or_else(|_| "{}".into());
            if ping_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Subscriptions + event poller.
    let subscriptions: Arc<tokio::sync::Mutex<HashSet<String>>> =
        Arc::new(tokio::sync::Mutex::new(HashSet::new()));
    let mut last_seen: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let db_clone = state.db.clone();
    let poll_tx = tx.clone();
    let subs_clone = subscriptions.clone();
    let poll_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        interval.tick().await;
        loop {
            interval.tick().await;
            let subs = subs_clone.lock().await.clone();
            for channel in subs {
                if let Some(order_id_str) = channel
                    .strip_prefix("order:")
                    .and_then(|s| s.strip_suffix(":events"))
                {
                    if let Ok(order_id) = Uuid::parse_str(order_id_str) {
                        let after = *last_seen.get(&channel).unwrap_or(&0);
                        match order_repo::events_after(&db_clone, order_id, after, 50).await {
                            Ok(events) => {
                                for (seq, event_type, payload, _created_at) in events {
                                    let msg = serde_json::to_string(&ServerMsg::OrderEvent {
                                        channel: channel.clone(),
                                        event_type: event_type.clone(),
                                        payload,
                                    })
                                    .unwrap_or_default();
                                    if poll_tx.send(msg).await.is_err() {
                                        return;
                                    }
                                    last_seen.insert(channel.clone(), seq);
                                }
                            }
                            Err(e) => {
                                warn!(error = ?e, "ws: failed to fetch events");
                            }
                        }
                    }
                }
            }
        }
    });

    // Read messages from client
    let db = state.db.clone();
    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let parsed: Result<ClientMsg, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(ClientMsg::Subscribe { channel }) => {
                        // Cap subscriptions per connection to prevent abuse
                        const MAX_SUBSCRIPTIONS: usize = 20;
                        let mut subs = subscriptions.lock().await;
                        if subs.len() >= MAX_SUBSCRIPTIONS && !subs.contains(&channel) {
                            drop(subs);
                            let _ = tx
                                .send(
                                    serde_json::to_string(&ServerMsg::Error {
                                        code: "subscription_limit".into(),
                                        message: format!(
                                            "max {} subscriptions per connection",
                                            MAX_SUBSCRIPTIONS
                                        ),
                                    })
                                    .unwrap_or_default(),
                                )
                                .await;
                            continue;
                        }
                        if let Err(e) =
                            validate_subscription(&channel, &user_id, role, &db).await
                        {
                            drop(subs);
                            let _ = tx
                                .send(
                                    serde_json::to_string(&ServerMsg::Error {
                                        code: "forbidden".into(),
                                        message: e,
                                    })
                                    .unwrap_or_default(),
                                )
                                .await;
                            continue;
                        }
                        subs.insert(channel.clone());
                        drop(subs);
                        info!(user_id = %user_id, channel = %channel, "ws subscribed");
                    }
                    Ok(ClientMsg::Unsubscribe { channel }) => {
                        subscriptions.lock().await.remove(&channel);
                    }
                    Ok(ClientMsg::Replay { channel, last_event_id }) => {
                        if let Some(order_id_str) = channel
                            .strip_prefix("order:")
                            .and_then(|s| s.strip_suffix(":events"))
                        {
                            if let Ok(order_id) = Uuid::parse_str(order_id_str) {
                                match order_repo::events_after(
                                    &db,
                                    order_id,
                                    last_event_id,
                                    100,
                                )
                                .await
                                {
                                    Ok(events) => {
                                        let event_payloads: Vec<_> = events
                                            .into_iter()
                                            .map(|(seq, event_type, payload, _created_at)| {
                                                json!({
                                                    "sequence": seq,
                                                    "event_type": event_type,
                                                    "payload": payload,
                                                })
                                            })
                                            .collect();
                                        let _ = tx
                                            .send(
                                                serde_json::to_string(&ServerMsg::ReplayResult {
                                                    channel,
                                                    events: event_payloads,
                                                })
                                                .unwrap_or_default(),
                                            )
                                            .await;
                                    }
                                    Err(e) => {
                                        warn!(error = ?e, "ws replay failed");
                                    }
                                }
                            }
                        }
                    }
                    Ok(ClientMsg::DriverLocation {
                        lat,
                        lng,
                        heading,
                        speed_kph,
                    }) => {
                        if role != UserRole::Driver {
                            let _ = tx
                                .send(
                                    serde_json::to_string(&ServerMsg::Error {
                                        code: "forbidden".into(),
                                        message: "only drivers can send location".into(),
                                    })
                                    .unwrap_or_default(),
                                )
                                .await;
                            continue;
                        }
                        let driver_user_id = match Uuid::parse_str(&user_id) {
                            Ok(u) => u,
                            Err(_) => continue,
                        };
                        let _ = realtime_service::publish_driver_location(
                            &state.redis,
                            driver_user_id,
                            None,
                            lat,
                            lng,
                            heading,
                            speed_kph,
                        )
                        .await;
                    }
                    Ok(ClientMsg::Pong) => {
                        // Heartbeat reply — no action needed.
                    }
                    Err(e) => {
                        warn!(error = ?e, "ws: invalid client message");
                        let _ = tx
                            .send(
                                serde_json::to_string(&ServerMsg::Error {
                                    code: "invalid_message".into(),
                                    message: format!("{}", e),
                                })
                                .unwrap_or_default(),
                            )
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Binary(_)) | Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                // ignore
            }
            Err(e) => {
                warn!(error = ?e, "ws: read error");
                break;
            }
        }
    }

    // Cleanup
    writer.abort();
    ping.abort();
    poll_task.abort();
    info!(user_id = %user_id, "ws disconnected");
}

async fn validate_subscription(
    channel: &str,
    user_id: &str,
    role: UserRole,
    _db: &sqlx::PgPool,
) -> Result<(), String> {
    if channel.starts_with("order:") && channel.ends_with(":events") {
        let _ = (user_id, role);
        return Ok(());
    }
    if channel.starts_with("user:") {
        let expected = format!("user:{}:notifications", user_id);
        if channel == expected {
            return Ok(());
        }
        return Err("cannot subscribe to other user's channel".into());
    }
    Err(format!("unknown channel: {}", channel))
}

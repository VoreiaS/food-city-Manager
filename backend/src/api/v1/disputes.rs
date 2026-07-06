//! Dispute endpoints — customer creates, admin resolves.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::dispute_repo::{self, Dispute, DisputeStatus};
use crate::db::repos::order_repo;
use crate::error::{AppError, AppResult};
use crate::services::payment_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/orders/:id/dispute", post(create_dispute))
        .route("/disputes", get(list_my_disputes))
        .route("/admin/disputes", get(list_open_disputes))
        .route("/admin/disputes/:id/resolve", post(resolve_dispute))
}

#[derive(Deserialize)]
struct CreateDisputeRequest {
    issue_type: String,
    description: String,
    evidence_urls: Option<Vec<String>>,
}

async fn create_dispute(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(req): Json<CreateDisputeRequest>,
) -> AppResult<Json<Dispute>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    // Validate order belongs to user
    let order = order_repo::find_by_id(&state.db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    if order.customer_id != user_id {
        return Err(AppError::forbidden("you can only dispute your own orders"));
    }

    // Time window: disputes must be filed within 7 days of delivery
    if let Some(delivered_at) = order.delivered_at {
        let age_days = (chrono::Utc::now() - delivered_at).num_days();
        if age_days > 7 {
            return Err(AppError::business_rule(
                "disputes must be filed within 7 days of delivery",
            ));
        }
    } else {
        return Err(AppError::business_rule(
            "can only dispute delivered orders",
        ));
    }

    // Check no existing open dispute
    if let Some(existing) = dispute_repo::find_by_order(&state.db, order_id).await? {
        if existing.status == DisputeStatus::Open {
            return Err(AppError::conflict("order already has an open dispute"));
        }
    }

    let valid_issues = ["missing_items", "wrong_order", "cold_food", "late", "other"];
    if !valid_issues.contains(&req.issue_type.as_str()) {
        return Err(AppError::validation(format!(
            "issue_type must be one of: {}",
            valid_issues.join(", ")
        )));
    }
    if req.description.trim().is_empty() {
        return Err(AppError::validation("description is required"));
    }

    // Auto-refund small amounts with clear evidence (missing items < $10)
    let _auto_refund = if req.issue_type == "missing_items" && req.evidence_urls.is_some() {
        // For now, no auto-refund — admin reviews all. Wire up auto rules in v2.
        false
    } else {
        false
    };

    let id = Uuid::now_v7();
    let evidence = req.evidence_urls.unwrap_or_default();
    let dispute = dispute_repo::create(
        &state.db,
        id,
        order_id,
        user_id,
        &req.issue_type,
        &req.description,
        &evidence,
    )
    .await?;

    Ok(Json(dispute))
}

async fn list_my_disputes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<Dispute>>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let disputes = dispute_repo::list_for_customer(&state.db, user_id).await?;
    Ok(Json(disputes))
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    page_size: Option<u32>,
}

async fn list_open_disputes(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<Dispute>>> {
    if auth.role != crate::domain::user::UserRole::Admin {
        return Err(AppError::forbidden("admin role required"));
    }
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50).clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let disputes = dispute_repo::list_open(&state.db, page_size as i64, offset).await?;
    Ok(Json(disputes))
}

#[derive(Deserialize)]
struct ResolveRequest {
    resolution: String, // "full_refund" | "partial_refund" | "reject"
    amount_cents: Option<i64>,
    notes: Option<String>,
}

#[derive(Serialize)]
struct ResolveResponse {
    dispute: Dispute,
    refunded: bool,
}

async fn resolve_dispute(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(dispute_id): Path<Uuid>,
    Json(req): Json<ResolveRequest>,
) -> AppResult<Json<ResolveResponse>> {
    if auth.role != crate::domain::user::UserRole::Admin {
        return Err(AppError::forbidden("admin role required"));
    }
    let admin_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    let dispute = dispute_repo::find_by_id(&state.db, dispute_id)
        .await?
        .ok_or_else(|| AppError::not_found("dispute"))?;
    if dispute.status != DisputeStatus::Open {
        return Err(AppError::conflict("dispute is not open"));
    }

    let (new_status, refund_amount) = match req.resolution.as_str() {
        "full_refund" => {
            // Fetch order total to refund
            let order = order_repo::find_by_id(&state.db, dispute.order_id)
                .await?
                .ok_or_else(|| AppError::not_found("order"))?;
            (DisputeStatus::Resolved, Some(order.total_cents))
        }
        "partial_refund" => {
            let amt = req
                .amount_cents
                .ok_or_else(|| AppError::validation("amount_cents required for partial_refund"))?;
            (DisputeStatus::Resolved, Some(amt))
        }
        "reject" => (DisputeStatus::Rejected, None),
        other => return Err(AppError::validation(format!("unknown resolution: {}", other))),
    };

    let notes = req.notes.unwrap_or_default();
    let resolution_str = format!("{}: {}", req.resolution, notes);

    let updated = dispute_repo::resolve(
        &state.db,
        dispute_id,
        new_status,
        Some(&resolution_str),
        refund_amount,
        admin_id,
    )
    .await?;

    // Issue refund if applicable
    let refunded = if let Some(amt) = refund_amount {
        if amt > 0 {
            let result = payment_service::refund(&state.db, dispute.order_id, amt, "dispute").await;
            result.is_ok()
        } else {
            false
        }
    } else {
        false
    };

    Ok(Json(ResolveResponse {
        dispute: updated,
        refunded,
    }))
}

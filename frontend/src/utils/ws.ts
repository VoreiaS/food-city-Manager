// WebSocket client with auto-reconnect + event replay.
//
// Usage:
//   const ws = new WsClient(token);
//   ws.subscribe(`order:${orderId}:events`, (msg) => { ... });
//   ws.disconnect();

type MessageHandler = (msg: Record<string, unknown>) => void;

export class WsClient {
  private ws: WebSocket | null = null;
  private url: string;
  private subscriptions = new Set<string>();
  private handlers = new Map<string, Set<MessageHandler>>();
  private lastEventId = new Map<string, number>();
  private reconnectAttempts = 0;
  private shouldReconnect = true;
  private reconnectTimer: number | null = null;
  private pingTimer: number | null = null;

  constructor(token: string) {
    const wsBase = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws";
    this.url = `${wsBase}?token=${encodeURIComponent(token)}`;
  }

  connect() {
    if (this.ws?.readyState === WebSocket.OPEN) return;
    try {
      this.ws = new WebSocket(this.url);
    } catch (e) {
      console.error("WS connect failed", e);
      this.scheduleReconnect();
      return;
    }
    this.ws.onopen = () => {
      console.debug("WS connected");
      this.reconnectAttempts = 0;
      this.startPing();
      // Re-subscribe to all channels
      for (const channel of this.subscriptions) {
        this.send({ type: "subscribe", channel });
        // Request replay of missed events
        const lastId = this.lastEventId.get(channel) ?? 0;
        if (lastId > 0) {
          this.send({ type: "replay", channel, last_event_id: lastId });
        }
      }
    };
    this.ws.onmessage = (e) => this.handleMessage(e);
    this.ws.onclose = () => {
      console.debug("WS closed");
      this.stopPing();
      if (this.shouldReconnect) {
        this.scheduleReconnect();
      }
    };
    this.ws.onerror = (e) => {
      console.error("WS error", e);
    };
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;
    const delay = Math.min(1000 * 2 ** this.reconnectAttempts, 30000);
    this.reconnectAttempts += 1;
    console.debug(`WS reconnect in ${delay}ms (attempt ${this.reconnectAttempts})`);
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }

  private startPing() {
    this.stopPing();
    this.pingTimer = window.setInterval(() => {
      this.send({ type: "pong" }); // server pings, we pong to confirm alive
    }, 25000);
  }

  private stopPing() {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  private send(msg: unknown) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  private handleMessage(e: MessageEvent) {
    let msg: Record<string, unknown>;
    try {
      msg = JSON.parse(e.data) as Record<string, unknown>;
    } catch {
      return;
    }
    if (msg.type === "order_event" && typeof msg.channel === "string") {
      // Track last event id per channel for replay.
      const channel = msg.channel as string;
      const payload = msg.payload as Record<string, unknown> | undefined;
      const seq = payload?.sequence;
      if (typeof seq === "number") {
        this.lastEventId.set(channel, Math.max(this.lastEventId.get(channel) ?? 0, seq));
      }
    }
    if (typeof msg.channel === "string") {
      const handlers = this.handlers.get(msg.channel);
      if (handlers) {
        handlers.forEach((h) => h(msg));
      }
    }
  }

  subscribe(channel: string, handler: MessageHandler) {
    if (!this.subscriptions.has(channel)) {
      this.subscriptions.add(channel);
      this.send({ type: "subscribe", channel });
    }
    if (!this.handlers.has(channel)) {
      this.handlers.set(channel, new Set());
    }
    const handlers = this.handlers.get(channel);
    if (handlers) handlers.add(handler);
  }

  unsubscribe(channel: string, handler?: MessageHandler) {
    if (handler) {
      this.handlers.get(channel)?.delete(handler);
    } else {
      this.handlers.delete(channel);
    }
    if (!handler || this.handlers.get(channel)?.size === 0) {
      this.subscriptions.delete(channel);
      this.send({ type: "unsubscribe", channel });
    }
  }

  disconnect() {
    this.shouldReconnect = false;
    this.stopPing();
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
  }
}

// Singleton instance (lazily connected on first subscribe)
let _client: WsClient | null = null;

export function getWsClient(token: string): WsClient {
  if (!_client) {
    _client = new WsClient(token);
    _client.connect();
  }
  return _client;
}

export function resetWsClient() {
  _client?.disconnect();
  _client = null;
}

# Food City — API Contract (v1)

> REST API + WebSocket protocol for Food City. All endpoints under `/api/v1`.
> Monetary values are integer cents. Timestamps are RFC 3339 UTC. IDs are
> ULID strings unless noted.

---

## Conventions

### Auth

- Bearer token: `Authorization: Bearer <jwt>`
- All endpoints except `/auth/register`, `/auth/login`, `/auth/refresh`,
  `/health`, public restaurant browsing, and Stripe webhook require auth.
- Role-based authorization per route (see `[role]` markers).

### Errors

```json
{
  "error": {
    "code": "validation_error",
    "message": "human-readable message",
    "details": { "field": "email", "issue": "already registered" },
    "request_id": "req_01HXYZ..."
  }
}
```

| HTTP | code | When |
|---|---|---|
| 400 | `validation_error` | Bad input |
| 401 | `unauthenticated` | Missing/invalid token |
| 403 | `forbidden` | Role mismatch |
| 404 | `not_found` | Resource doesn't exist |
| 409 | `conflict` | State transition invalid |
| 422 | `business_rule_violation` | Domain rule failed (e.g., promo exhausted) |
| 429 | `rate_limited` | Too many requests |
| 500 | `internal_error` | Unhandled |
| 503 | `service_unavailable` | Circuit breaker open / maintenance |

### Pagination

```
GET /restaurants?page=2&page_size=20
→ { "data": [...], "page": 2, "page_size": 20, "total": 137 }
```

### Common headers

- `X-Request-Id` — generated client-side or by gateway; propagated to logs.
- `Idempotency-Key` — required on `POST /orders`, `POST /payments/intent`.

---

## 1. Auth

### POST `/auth/register`

```json
{
  "email": "user@example.com",
  "phone": "+15551234567",
  "password": "••••••••",
  "full_name": "Jane Doe",
  "role": "customer"  // customer | restaurant | driver | admin
}
```

**201 Created**
```json
{
  "user": { "id": "01H...", "email": "...", "role": "customer" },
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "expires_in": 900
}
```

### POST `/auth/login`

```json
{ "email": "user@example.com", "password": "••••••••" }
```

**200 OK** — same response as register.

### POST `/auth/refresh`

```json
{ "refresh_token": "eyJ..." }
```

**200 OK** — new access + refresh tokens.

### POST `/auth/logout`

(Requires auth.) Invalidates refresh token.

### GET `/auth/me`

(Requires auth.) Returns current user profile.

---

## 2. Addresses

### GET `/addresses` — list user's addresses
### POST `/addresses` — add new (requires OTP if first time on device)
### PATCH `/addresses/:id` — update
### DELETE `/addresses/:id` — soft delete

```json
{
  "id": "01H...",
  "label": "Home",
  "line1": "123 Main St",
  "line2": "Apt 4B",
  "city": "Colombo",
  "lat": 6.9271,
  "lng": 79.8612,
  "formatted_address": "123 Main St, Colombo",
  "is_default": true
}
```

---

## 3. Restaurants

### GET `/restaurants`

Query params:
- `lat`, `lng` (required) — center point
- `radius_m` (default 5000)
- `cuisine` (comma-separated)
- `price_range` (1-4)
- `veg_only` (bool)
- `rating_min` (1-5)
- `sort` (`distance` | `rating` | `eta` | `promos`)
- `q` (search term — matches name + cuisine + dish name)
- `page`, `page_size`

**200 OK**
```json
{
  "data": [
    {
      "id": "01H...",
      "name": "Spice Villa",
      "slug": "spice-villa",
      "cuisine_types": ["indian", "vegetarian"],
      "price_range": 2,
      "rating_avg": 4.6,
      "rating_count": 1243,
      "logo_url": "https://...",
      "cover_url": "https://...",
      "delivery_fee_cents": 250,
      "delivery_eta_min": 25,
      "distance_m": 1200,
      "is_open": true,
      "promos": [
        { "code": "SPICE20", "description": "20% off" }
      ]
    }
  ],
  "page": 1, "page_size": 20, "total": 137
}
```

### GET `/restaurants/:id`

**200 OK** — full restaurant detail including hours, full promo list.

### GET `/restaurants/:id/menu`

**200 OK**
```json
{
  "restaurant_id": "01H...",
  "menu_version": 7,
  "categories": [
    {
      "id": "01H...",
      "name": "Starters",
      "sort_order": 1,
      "items": [
        {
          "id": "01H...",
          "name": "Paneer Tikka",
          "description": "...",
          "price_cents": 850,
          "image_url": "https://...",
          "is_veg": true,
          "spice_level": 2,
          "allergens": ["dairy"],
          "track_stock": false,
          "in_stock": true,
          "customizations": [
            {
              "id": "01H...",
              "name": "Spice level",
              "options": [
                { "name": "Mild", "price_cents": 0, "is_default": true },
                { "name": "Medium", "price_cents": 0 },
                { "name": "Hot", "price_cents": 50 }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

### POST `/restaurants` `[restaurant | admin]`

Restaurant onboarding (KYC docs).

### PATCH `/restaurants/:id` `[restaurant owner | admin]`

Update profile (hours, photos, status).

### POST `/restaurants/:id/menu/versions` `[restaurant owner]`

Publish new menu version.

### PATCH `/restaurants/:id/menu/items/:itemId` `[restaurant owner]`

Live updates: price (creates new version), availability (no version
bump), stock.

---

## 4. Cart

### GET `/cart`

Returns current active cart for authenticated user. Empty if none.

### POST `/cart/items`

```json
{
  "restaurant_id": "01H...",
  "menu_item_id": "01H...",
  "quantity": 2,
  "customizations": [
    { "customization_id": "01H...", "option_name": "Hot" }
  ],
  "notes": "Extra chutney please"
}
```

If cart exists for a different restaurant, returns **409 conflict**
with `details: { current_restaurant_id, requested_restaurant_id }`.
Client must confirm clearing cart first.

**200 OK** — updated cart.

### PATCH `/cart/items/:itemId`

```json
{ "quantity": 3, "notes": "Updated note" }
```

### DELETE `/cart/items/:itemId`

### DELETE `/cart` — clear cart

---

## 5. Orders

### POST `/orders`

**Headers:** `Idempotency-Key: <uuid>`

```json
{
  "address_id": "01H...",
  "payment_method_id": "pm_..." | null,  // null = use new payment
  "promo_code": "WELCOME50" | null,
  "tip_cents": 300,
  "loyalty_points_to_redeem": 0,
  "notes": "Leave at door"
}
```

**Workflow:**
1. Validate cart is non-empty, address in range, restaurant open.
2. Snapshot cart → order + order_items.
3. Validate promo (atomic increment).
4. Create payment intent (idempotent).
5. Return order + client_secret for Stripe.

**201 Created**
```json
{
  "order": {
    "id": "01H...",
    "status": "pending_accept",
    "total_cents": 4550,
    "currency": "usd",
    "estimated_delivery_at": "2026-07-05T13:30:00Z"
  },
  "payment": {
    "intent_id": "pi_...",
    "client_secret": "pi_..._secret_...",
    "status": "requires_confirmation"
  }
}
```

### GET `/orders` — list user's orders
### GET `/orders/:id` — single order detail

### POST `/orders/:id/cancel`

```json
{ "reason": "changed_mind" }
```

Returns **409** if cancellation window elapsed; includes refund amount
that *would* be given if admin approves.

### POST `/orders/:id/dispute`

```json
{
  "issue_type": "missing_items" | "wrong_order" | "cold_food" | "late" | "other",
  "description": "...",
  "evidence_urls": ["https://..."]
}
```

---

## 6. Order Realtime (WebSocket)

### Connection

`GET wss://api.foodcity.app/ws?token=<jwt>`

On connect, server validates JWT and sends:
```json
{ "type": "connected", "user_id": "01H...", "server_time": "..." }
```

### Client → Server messages

```json
// Subscribe to a channel
{ "type": "subscribe", "channel": "order:01H..." }

// Unsubscribe
{ "type": "unsubscribe", "channel": "order:01H..." }

// Reconnect replay
{ "type": "replay", "channel": "order:01H...", "last_event_id": 42 }

// Driver location update (driver role only)
{ "type": "driver_location", "lat": 6.9, "lng": 79.8, "heading": 90, "speed_kph": 30 }
```

### Server → Client messages

```json
// Order state change
{
  "type": "order_event",
  "channel": "order:01H...",
  "event_id": 43,
  "event_type": "status_changed",
  "sequence": 43,
  "payload": {
    "order_id": "01H...",
    "status": "preparing",
    "occurred_at": "..."
  }
}

// Driver location
{
  "type": "driver_location",
  "channel": "order:01H...",
  "event_id": 44,
  "payload": { "lat": 6.9, "lng": 79.8, "eta_min": 8 }
}

// Driver assigned
{
  "type": "order_event",
  "event_type": "driver_assigned",
  "payload": {
    "driver": { "id": "01H...", "name": "John", "photo_url": "...", "rating": 4.9, "vehicle": "Bike" }
  }
}

// Replay response
{ "type": "replay_result", "channel": "order:...", "events": [...] }
// OR snapshot fallback
{ "type": "snapshot", "channel": "order:...", "state": { ... } }

// Error
{ "type": "error", "code": "invalid_channel", "message": "..." }
```

### Heartbeat

- Server sends `{"type":"ping"}` every 30s.
- Client must respond `{"type":"pong"}` within 60s or be disconnected.

---

## 7. Driver

### POST `/drivers/me/online` `[driver]`

Toggle status to `available`. Starts heartbeat + location push.

### POST `/drivers/me/offline` `[driver]`

### POST `/drivers/me/location` `[driver]`

```json
{ "lat": 6.9271, "lng": 79.8612, "heading": 90, "speed_kph": 30 }
```

### GET `/drivers/me/orders/offers` `[driver]`

Current pending offers (if broadcast model). Long-poll or WS-driven.

### POST `/drivers/orders/:orderId/accept` `[driver]`

Atomic transition `available → assigned`. Returns **409** if already
taken.

### POST `/drivers/orders/:orderId/pickup` `[driver]`

Mark `picked_up`. Body: `{ "otp": "1234" }` (if OTP required).

### POST `/drivers/orders/:orderId/deliver` `[driver]`

Mark `delivered`. Body:
```json
{ "otp": "1234", "photo_url": "https://...", "lat": 6.9, "lng": 79.8 }
```

### GET `/drivers/me/earnings` `[driver]`

Daily/weekly summary.

---

## 8. Restaurant (dashboard)

### GET `/restaurant/orders` `[restaurant]`

Active + recent orders. Filter by status, date.

### POST `/restaurant/orders/:id/accept` `[restaurant]`
### POST `/restaurant/orders/:id/reject` `[restaurant]`

Body for reject: `{ "reason": "out_of_ingredient" | "too_busy" | "closing" | "other" }`

### POST `/restaurant/orders/:id/preparing` `[restaurant]`
### POST `/restaurant/orders/:id/ready` `[restaurant]`

### PATCH `/restaurant/orders/:id/items/:itemId` `[restaurant]`

Update item status: `out_of_stock` (triggers customer notification +
partial refund option).

### GET `/restaurant/menu` `[restaurant]`
### POST `/restaurant/menu/items` `[restaurant]`
### PATCH `/restaurant/menu/items/:id` `[restaurant]`
### POST `/restaurant/menu/publish` `[restaurant]` — publish draft menu as new version

### GET `/restaurant/reviews` `[restaurant]`
### POST `/restaurant/reviews/:id/reply` `[restaurant]`

### GET `/restaurant/earnings` `[restaurant]`

---

## 9. Reviews (public)

### POST `/reviews`

```json
{
  "order_id": "01H...",
  "rating_food": 5,
  "rating_delivery": 4,
  "rating_packaging": 5,
  "rating_overall": 5,
  "body": "Great food!",
  "photo_urls": ["https://..."]
}
```

**Constraints:** customer must have completed order at restaurant;
one review per order.

### GET `/restaurants/:id/reviews`

Paginated, filterable by rating, with reply.

---

## 10. Loyalty

### GET `/loyalty/me` `[customer]`

```json
{
  "points_balance": 1250,
  "tier": "gold",
  "lifetime_points": 5420,
  "next_tier_points": 10000,
  "tier_benefits": ["free_delivery", "priority_support"]
}
```

### GET `/loyalty/me/transactions` `[customer]`

### POST `/loyalty/redeem` `[customer]` — at checkout (called by order service)

---

## 11. Payments

### POST `/payments/intents`

**Headers:** `Idempotency-Key: <uuid>`

```json
{
  "order_id": "01H...",
  "payment_method": "stripe",
  "save_payment_method": true
}
```

**201 Created**
```json
{
  "intent_id": "pi_...",
  "client_secret": "pi_..._secret_...",
  "status": "requires_confirmation",
  "amount_cents": 4550,
  "currency": "usd"
}
```

### POST `/payments/webhooks/stripe`

Stripe-signed webhook. Handles:
- `payment_intent.succeeded`
- `payment_intent.payment_failed`
- `charge.refunded`
- `charge.dispute.created` / `charge.dispute.closed`
- `transfer.created` (payout to restaurant/driver)

Returns **200** on success, **5xx** to trigger retry. Idempotent via
event ID.

### POST `/payments/refunds`

```json
{
  "order_id": "01H...",
  "amount_cents": 2000,  // partial refund allowed
  "reason": "missing_items"
}
```

`[admin]` for full amount; `[customer]` only via dispute flow.

---

## 12. Admin

### GET `/admin/verifications` `[admin]`
### POST `/admin/verifications/:id/approve` `[admin]`
### POST `/admin/verifications/:id/reject` `[admin]`

### GET `/admin/live/orders` `[admin]` — city-wide active orders
### GET `/admin/live/drivers` `[admin]` — available + active drivers
### GET `/admin/live/restaurants` `[admin]` — open/closed/paused

### POST `/admin/orders/:id/reassign` `[admin]`

Body: `{ "new_driver_id": "01H..." }`

### GET `/admin/disputes` `[admin]`
### POST `/admin/disputes/:id/resolve` `[admin]`

```json
{ "resolution": "full_refund" | "partial_refund" | "reject", "amount_cents": 2000, "notes": "..." }
```

### GET `/admin/promos` `[admin]`
### POST `/admin/promos` `[admin]`
### PATCH `/admin/promos/:id` `[admin]`

### GET `/admin/analytics/gmv` `[admin]`
### GET `/admin/analytics/retention` `[admin]`

---

## 13. Health

### GET `/health`

```json
{ "status": "ok", "version": "0.1.0" }
```

### GET `/ready`

```json
{ "db": "ok", "redis": "ok", "stripe": "ok" }
```

Returns **503** if any dependency down.

---

## Rate Limits

| Endpoint | Limit | Burst |
|---|---|---|
| `POST /auth/login` | 5/min/IP | 10 |
| `POST /auth/register` | 3/hour/IP | 5 |
| `POST /orders` | 10/min/user | 20 |
| `POST /reviews` | 5/min/user | 10 |
| All others | 100/min/user | 200 |
| Anonymous (per IP) | 60/min | 120 |

Headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`.


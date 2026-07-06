# Food City — Architecture

> System architecture for a Zomato/Swiggy-style restaurant marketplace with
> full delivery: Rust + Axum backend, React + Vite + Tailwind frontend,
> PostgreSQL + Redis, Stripe Connect payments.

---

## 1. High-Level Architecture

```
                          ┌─────────────────────────────────────────┐
                          │            Client Tier                  │
                          │  ┌──────────┐  ┌──────────┐  ┌────────┐ │
                          │  │ Customer │  │ Restaurant│  │ Driver │ │
                          │  │   Web    │  │ Dashboard │  │  App   │ │
                          │  └────┬─────┘  └─────┬─────┘  └───┬────┘ │
                          │       │              │             │      │
                          │       └──────┬───────┴─────────────┘      │
                          │              │  Admin Web (Ops Console)   │
                          └──────────────┼───────────────────────────┘
                                         │
                          HTTPS (REST) + WSS (realtime)
                                         │
                          ┌──────────────▼───────────────────┐
                          │       API Gateway (Axum)          │
                          │  ┌─────────────────────────────┐  │
                          │  │  Auth Middleware (JWT)       │  │
                          │  │  Rate Limiter (Redis token)  │  │
                          │  │  Request ID + Tracing        │  │
                          │  │  CORS                        │  │
                          │  └─────────────────────────────┘  │
                          │                                   │
                          │  ┌──────┐ ┌──────┐ ┌──────┐ ┌────┐│
                          │  │ Auth │ │Rest- │ │Order │ │ WS ││
                          │  │ API  │ │aurants│ │ API  │ │Gate││
                          │  └──┬───┘ └──┬───┘ └──┬───┘ └─┬──┘│
                          │     │        │        │       │    │
                          │  ┌──▼────────▼────────▼─────┐ │    │
                          │  │   Service Layer          │ │    │
                          │  │  (business logic, no IO) │ │    │
                          │  └──┬────────┬────────┬────┘ │    │
                          │     │        │        │      │    │
                          │  ┌──▼───┐ ┌──▼──┐ ┌───▼───┐  │    │
                          │  │ Repo │ │Cache│ │Events │  │    │
                          │  │Layer │ │ Svc │ │  Svc  │  │    │
                          │  └──┬───┘ └──┬──┘ └───┬───┘  │    │
                          └─────┼─────────┼─────────┼──────┼────┘
                                │         │         │      │
                          ┌─────▼────┐ ┌──▼─────┐ ┌─▼──────▼──┐
                          │PostgreSQL│ │  Redis │ │  Workers  │
                          │ Primary  │ │        │ │ (background│
                          │  + Read  │ │ PubSub │ │  jobs)    │
                          │ Replica  │ │  GEO   │ └───────────┘
                          └──────────┘ │ Streams│
                                       └────────┘
                                │
                          ┌─────▼──────┐
                          │  Stripe    │
                          │  Connect   │
                          └────────────┘
```

---

## 2. Backend Module Map

```
backend/
├── Cargo.toml
├── .env.example
├── migrations/
│   ├── 20260101000000_create_users.sql
│   ├── 20260101000001_create_restaurants.sql
│   ├── 20260101000002_create_menus.sql
│   ├── 20260101000003_create_orders.sql
│   ├── 20260101000004_create_drivers.sql
│   ├── 20260101000005_create_reviews.sql
│   ├── 20260101000006_create_loyalty.sql
│   └── 20260101000007_create_payments.sql
└── src/
    ├── main.rs                    # Binary entry, server bootstrap
    ├── lib.rs                     # Re-exports for integration tests
    ├── config.rs                  # Env config (figment/envy)
    ├── error.rs                   # AppError + IntoResponse
    │
    ├── api/                       # HTTP handlers (thin)
    │   ├── mod.rs
    │   ├── v1/
    │   │   ├── mod.rs             # Router composition
    │   │   ├── auth.rs            # POST /register /login /refresh
    │   │   ├── restaurants.rs     # GET /restaurants /:id
    │   │   ├── menus.rs           # GET /restaurants/:id/menu
    │   │   ├── cart.rs            # GET/POST/DELETE /cart
    │   │   ├── orders.rs          # POST /orders, GET /orders/:id
    │   │   ├── drivers.rs         # Driver state, location
    │   │   ├── reviews.rs         # POST/GET reviews
    │   │   ├── loyalty.rs         # Points, tiers, redemption
    │   │   ├── payments.rs        # Intents, webhooks
    │   │   ├── admin.rs           # Admin-only endpoints
    │   │   └── ws.rs              # WS upgrade + handler
    │   └── mw/                    # Middlewares
    │       ├── auth.rs            # Extract authenticated user
    │       ├── rate_limit.rs      # Per-IP / per-user rate limit
    │       ├── request_id.rs      # X-Request-Id propagation
    │       └── trace.rs           # OpenTelemetry spans
    │
    ├── domain/                    # Pure domain types (no IO)
    │   ├── mod.rs
    │   ├── user.rs
    │   ├── restaurant.rs
    │   ├── menu.rs
    │   ├── order.rs               # State machine definitions
    │   ├── driver.rs
    │   ├── review.rs
    │   ├── payment.rs
    │   └── loyalty.rs
    │
    ├── services/                  # Business logic (calls repos)
    │   ├── mod.rs
    │   ├── auth_service.rs        # Hash, JWT issue/verify
    │   ├── restaurant_service.rs
    │   ├── menu_service.rs        # Versioning, scheduling
    │   ├── order_service.rs       # Cart snapshot, state transitions
    │   ├── driver_service.rs      # Matching, location smoothing
    │   ├── payment_service.rs     # Stripe, idempotency, webhooks
    │   ├── review_service.rs
    │   ├── loyalty_service.rs     # Points accrual, redemption
    │   ├── geo_service.rs         # Distance, geofence, ETA
    │   ├── realtime_service.rs    # WS fan-out, Redis pub/sub
    │   └── cache_service.rs       # Singleflight, stale-while-revalidate
    │
    ├── db/                        # Persistence layer
    │   ├── mod.rs
    │   ├── pool.rs                # Primary + replica pools
    │   └── repos/
    │       ├── mod.rs
    │       ├── user_repo.rs
    │       ├── restaurant_repo.rs
    │       ├── menu_repo.rs
    │       ├── order_repo.rs
    │       ├── driver_repo.rs
    │       ├── review_repo.rs
    │       ├── payment_repo.rs
    │       └── loyalty_repo.rs
    │
    ├── workers/                   # Background jobs
    │   ├── mod.rs
    │   ├── order_acceptance_timeout.rs
    │   ├── driver_match_loop.rs
    │   ├── delivery_eta_recalc.rs
    │   ├── driver_pickup_watchdog.rs
    │   ├── payment_reconciler.rs
    │   ├── payout_scheduler.rs    # Weekly payouts
    │   └── anomaly_detector.rs    # Fraud signals
    │
    └── utils/
        ├── jwt.rs
        ├── hash.rs                # argon2
        ├── geo.rs                 # Haversine fallback
        └── id.rs                  # ULID/UUID generation
```

### Layering rules (enforced by lint / review)

- **`api/`** may only call `services/` and `db::repos/` (read-only convenience queries OK).
- **`services/`** may call `db::repos/`, other services, `utils/`. May not know about HTTP (`axum`, `http`).
- **`db::repos/`** may only call `sqlx` + `domain/`. Returns domain types.
- **`domain/`** has zero external deps beyond `serde`, `chrono`, `uuid`.
- **`workers/`** may call `services/` like an API handler would.

This separation makes the business logic testable without HTTP/DB mocks.

---

## 3. Frontend Module Map

```
frontend/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── postcss.config.js
├── index.html
├── .env.example
├── public/
└── src/
    ├── main.tsx                   # App bootstrap
    ├── App.tsx                    # Providers (QueryClient, Auth, Router)
    ├── router.tsx                 # Route definitions
    │
    ├── api/                       # API client
    │   ├── client.ts              # fetch wrapper + interceptors
    │   ├── auth.api.ts
    │   ├── restaurants.api.ts
    │   ├── menu.api.ts
    │   ├── cart.api.ts
    │   ├── orders.api.ts
    │   ├── drivers.api.ts
    │   ├── reviews.api.ts
    │   ├── loyalty.api.ts
    │   └── payments.api.ts
    │
    ├── hooks/                     # React Query hooks
    │   ├── useAuth.ts
    │   ├── useRestaurants.ts
    │   ├── useCart.ts
    │   ├── useOrder.ts
    │   ├── useOrderTracking.ts    # WS subscription
    │   └── useLoyalty.ts
    │
    ├── store/                     # Client state (Zustand)
    │   ├── authStore.ts
    │   ├── cartStore.ts           # Optimistic cart UI
    │   └── uiStore.ts             # Theme, modals, toasts
    │
    ├── components/
    │   ├── ui/                    # Primitives: Button, Input, Modal
    │   ├── layout/                # Header, Footer, Sidebar
    │   ├── restaurant/            # RestaurantCard, MenuList
    │   ├── cart/                  # CartDrawer, CartItem
    │   ├── order/                 # OrderTracker, StatusTimeline
    │   ├── map/                   # Leaflet wrapper, DriverMarker
    │   └── common/                # EmptyState, ErrorBoundary, Spinner
    │
    ├── pages/
    │   ├── customer/
    │   │   ├── HomePage.tsx
    │   │   ├── RestaurantPage.tsx
    │   │   ├── CheckoutPage.tsx
    │   │   ├── OrderTrackingPage.tsx
    │   │   ├── OrdersPage.tsx
    │   │   ├── ProfilePage.tsx
    │   │   └── LoyaltyPage.tsx
    │   ├── restaurant/
    │   │   ├── DashboardPage.tsx
    │   │   ├── OrdersPage.tsx
    │   │   ├── MenuPage.tsx
    │   │   ├── ReviewsPage.tsx
    │   │   └── EarningsPage.tsx
    │   ├── driver/
    │   │   ├── ShiftPage.tsx
    │   │   ├── OrderOfferPage.tsx
    │   │   ├── ActiveDeliveryPage.tsx
    │   │   └── EarningsPage.tsx
    │   ├── admin/
    │   │   ├── LiveOpsPage.tsx
    │   │   ├── VerificationsPage.tsx
    │   │   ├── DisputesPage.tsx
    │   │   └── AnalyticsPage.tsx
    │   └── auth/
    │       ├── LoginPage.tsx
    │       └── RegisterPage.tsx
    │
    ├── types/                     # Shared TS types (mirror domain/)
    │   ├── user.ts
    │   ├── restaurant.ts
    │   ├── order.ts
    │   └── ...
    │
    ├── utils/
    │   ├── format.ts              # Currency, date, distance
    │   ├── geo.ts                 # Client-side distance calc
    │   └── ws.ts                  # WS client with reconnect
    │
    └── styles/
        └── index.css              # Tailwind directives
```

---

## 4. Data Model (high-level)

### Core entities

- **users** — id, email, phone, password_hash, role (enum: customer/restaurant/driver/admin), created_at, deleted_at
- **addresses** — id, user_id, label, line1, line2, city, lat, lng, formatted_address, is_default
- **restaurants** — id, owner_user_id, name, slug, group_id, cuisine_types, price_range, logo_url, cover_url, geom (GEOGRAPHY POINT), delivery_radius_m, status, hours_json, rating_avg, rating_count
- **restaurant_groups** — id, name, logo_url (for chains)
- **restaurant_hours_exceptions** — restaurant_id, date, is_closed, open_time, close_time
- **menu_versions** — id, restaurant_id, version, published_at (immutable once published)
- **menu_categories** — id, menu_version_id, name, sort_order
- **menu_items** — id, category_id, name, description, price_cents, image_url, is_veg, allergens, spice_level, track_stock, stock_count, sort_order, status
- **menu_item_customizations** — id, item_id, name, options (JSONB: [{name, price_cents, is_default}])
- **carts** — id, user_id, restaurant_id, status (active/locked/converted/abandoned), created_at
- **cart_items** — id, cart_id, menu_item_id, menu_version_at_add, quantity, customizations JSONB, notes
- **orders** — id, customer_id, restaurant_id, driver_id, status, payment_status, snapshot JSONB, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents, total_cents, currency, delivery_address JSONB, placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at, cancellation_reason
- **order_items** — id, order_id, menu_item_id (snapshot), name, price_cents, quantity, customizations JSONB, status
- **order_events** — id BIGSERIAL, order_id, sequence, event_type, payload JSONB, created_at (for WS replay)
- **drivers** — id, user_id, vehicle_type, license_plate, current_lat, current_lng, status (offline/available/assigned/en_route/at_restaurant/picked_up/delivering/delivered), current_order_id, rating_avg, acceptance_rate
- **driver_heartbeats** — Redis-only (key: `driver:hb:{id}`, TTL 60s)
- **driver_locations** — Redis GEO set `drivers:locations` for hot path; long-term in `driver_location_history` (Timescale or batched PG inserts)
- **reviews** — id, order_id UNIQUE, customer_id, restaurant_id, rating_food, rating_delivery, rating_packaging, rating_overall, body, photos JSONB, reply_body, reply_at, created_at
- **payment_intents** — id, order_id, provider, provider_intent_id, idempotency_key UNIQUE, amount_cents, currency, status, created_at
- **payment_webhooks** — id, provider_event_id UNIQUE, event_type, payload JSONB, processed_at
- **payout_ledger** — id, order_id, payee_type (restaurant/driver/platform), payee_id, amount_cents, stripe_transfer_id, status, created_at
- **disputes** — id, order_id, customer_id, issue_type, description, evidence_urls JSONB, status (open/resolved/rejected), resolution, refund_amount_cents, created_at, resolved_at
- **promo_codes** — id, code UNIQUE, discount_type, discount_value, min_order_cents, max_uses, used_count, daily_cap, per_user_cap, valid_from, valid_until, active
- **promo_redemptions** — id, promo_code_id, user_id, order_id, redeemed_at (UNIQUE (promo_code_id, user_id))
- **loyalty_accounts** — id, user_id, points_balance, tier (silver/gold/platinum), lifetime_points
- **loyalty_transactions** — id, account_id, points_delta, reason (order/refund/redemption), order_id, created_at
- **delivery_proofs** — id, order_id, photo_url, gps_lat, gps_lng, otp_hash, delivered_at
- **restaurant_verifications** — id, restaurant_id, status, documents JSONB, reviewed_by, reviewed_at, notes

### Key constraints

- `orders.status` transitions guarded by partial indices + service-layer checks
- `payment_intents.idempotency_key` UNIQUE
- `reviews.order_id` UNIQUE
- `(promo_code_id, user_id)` UNIQUE on `promo_redemptions`
- Soft delete (`deleted_at`) on users, restaurants, drivers
- All monetary amounts stored as `BIGINT` cents (no floats)

---

## 5. Realtime Architecture

### WebSocket topology

```
[Client WS] ⇄ [WS Gateway (Axum)] ⇄ [Redis PubSub]
                    │
                    ├── Subscribes to: order:{id}:events (per connected client)
                    ├── Subscribes to: user:{id}:notifications
                    └── Publishes: driver location updates (driver app only)
```

### Connection lifecycle

1. Client opens WSS to `/ws?token=JWT`
2. Server validates JWT, extracts user_id + role
3. Server subscribes to user's notification channel
4. Client sends `subscribe` messages: `{"type":"subscribe","channel":"order:123"}`
5. Server adds channel to per-connection subscription set
6. Redis pub/sub messages fan out to subscribed clients
7. Client sends `last_event_id` on (re)connect for replay
8. Heartbeat: server pings every 30s; client must pong within 60s

### Scaling

- Each WS gateway instance holds up to ~10k connections
- Redis pub/sub syncs events across instances (no sticky sessions needed)
- Connection draining on deploy: stop accepting new, wait 30s for close
- Per-instance metrics: connections, messages/sec, replay queue depth

---

## 6. Payment Architecture (Stripe Connect)

```
[Customer pays]
  └─> Stripe PaymentIntent (amount = total)
      └─> Webhook: payment_intent.succeeded
          └─> Update order.payment_status = paid
          └─> Trigger order acceptance flow
          └─> Schedule payout split

[On delivery complete]
  └─> payout_scheduler job:
      ├─> Stripe Transfer → restaurant connected account (subtotal − commission)
      ├─> Stripe Transfer → driver connected account (delivery fee + tip)
      └─> Platform retains: commission
```

### Stripe Connect account types

- **Restaurants**: `Express` accounts (Stripe handles KYC UI)
- **Drivers**: `Express` accounts with weekly payouts (or instant for 1% fee)

### Webhook security

- Stripe signature verification on every webhook
- Idempotency via `payment_webhooks.provider_event_id UNIQUE`
- Webhooks return 2xx only after DB commit

---

## 7. Caching Strategy

| Cache | TTL | Invalidation | Storage |
|---|---|---|---|
| Restaurant list (per city) | 30s | Time-based | Redis |
| Restaurant detail | 60s | On restaurant update | Redis |
| Menu (per restaurant) | 60s | On menu publish | Redis |
| User profile | 5min | On profile update | Redis |
| Available drivers (geo set) | persistent | On driver state change | Redis GEO |
| Driver location | persistent | On every ping (5s) | Redis GEO |
| Promo code validation | 5min | On redemption / disable | Redis |
| Rate limit counters | sliding window | n/a | Redis |

### Cache stampede prevention (EC-8.6)

- Singleflight: Redis `SETNX lock:cache:{key} EX 5` — only first miss fills
- Stale-while-revalidate: serve stale value while background refresher runs
- Probabilistic early expiration: random 0-30s offset on TTL

---

## 8. Observability

- **Tracing**: `tracing` + `tracing-subscriber` with OpenTelemetry exporter
- **Metrics**: Prometheus exporter (`/metrics` endpoint) — request latency, error rate, DB pool size, WS connections, queue depth
- **Structured logs**: JSON to stdout, correlation via `trace_id` + `request_id`
- **Health checks**: `/health` (liveness) + `/ready` (readiness, checks DB + Redis)

### Key metrics

- `http_requests_total{route, status}`
- `http_request_duration_seconds{route}`
- `db_pool_connections{state=active|idle}`
- `ws_connections_active`
- `order_state_count{state}` (gauge)
- `driver_state_count{state}` (gauge)
- `payment_intent_count{status}`
- `queue_depth{name}`

---

## 9. Security

- **AuthN**: JWT (access 15min + refresh 7d), argon2 password hashing
- **AuthZ**: Role-based (`customer` / `restaurant` / `driver` / `admin`), per-route middleware
- **Input validation**: `validator` crate on all DTOs; serde strict mode
- **SQL injection**: `sqlx` prepared statements everywhere (no string concat)
- **XSS**: React default escaping; CSP headers; sanitize uploaded HTML (reviews)
- **CSRF**: SameSite=Lax cookies + custom header for state-changing requests
- **Rate limiting**: Per-IP (global) + per-user (sensitive endpoints)
- **Secrets**: Env vars only; `.env` in `.gitignore`; production via secret manager
- **File uploads** (review photos, menu images): signed URLs (S3-compatible), virus scan on upload
- **PII**: Phone/email encrypted at rest in `users` table (application-level AES)
- **GDPR**: Soft-delete → hard-delete after 30 days; export endpoint for user data

---

## 10. Deployment

### Docker Compose (dev)

```yaml
services:
  postgres:    # PG 16 + PostGIS
  redis:       # Redis 7
  backend:     # Rust app
  frontend:    # Vite dev server
  ws_gateway:  # Optional: separate WS instance for scale testing
```

### Production

- **Backend**: Docker image → Kubernetes (or Fly.io / Railway)
- **Frontend**: Vite build → CDN (Vercel / Cloudflare Pages)
- **DB**: Managed PostgreSQL (RDS / Cloud SQL) with read replica
- **Redis**: Managed Redis (ElastiCache / Upstash)
- **Stripe**: Webhooks via Stripe's signed requests; no inbound ports needed
- **Maps**: Mapbox / OpenStreetMap (Leaflet on frontend)

### CI/CD pipeline

1. Push to `main` → GitHub Actions
2. Run `cargo test` + `cargo clippy` + `cargo fmt --check`
3. Run `npm test` + `npm run build` + `tsc --noEmit`
4. Build Docker images, push to registry
5. Deploy to staging; run smoke tests
6. Manual promote to production

---

## 11. Tech Stack Summary

| Layer | Technology | Why |
|---|---|---|
| Backend language | Rust 1.96+ | Performance, memory safety, async |
| Web framework | Axum 0.7+ | Tokio-native, tower middleware, typed extractors |
| Async runtime | Tokio | Industry standard, ecosystem |
| DB driver | sqlx 0.8 | Compile-time checked queries, pool |
| Migration | sqlx::migrate! | Same toolchain as queries |
| Cache / pub-sub | Redis (deadpool-redis) | Industry standard |
| Auth | jsonwebtoken + argon2 | Standards-based |
| Validation | validator | Serde integration |
| Error handling | thiserror + anyhow | Ergonomic + typed |
| Tracing | tracing + opentelemetry | Production-grade |
| Frontend framework | React 18 | Ecosystem, familiarity |
| Build tool | Vite 5 | Fast HMR, ESM-native |
| Language | TypeScript 5 | Type safety |
| Styling | Tailwind CSS 3 | Utility-first, fast iteration |
| Data fetching | TanStack Query 5 | Cache, retries, optimistic updates |
| Routing | React Router 6 | Standard, declarative |
| State | Zustand | Lightweight, no boilerplate |
| Forms | React Hook Form + Zod | Validation, performance |
| Maps | Leaflet + React-Leaflet | OSS, no per-load pricing |
| Payments | Stripe + Stripe Connect | Multi-party payouts |
| Realtime | Native WebSocket | Axum has built-in support |


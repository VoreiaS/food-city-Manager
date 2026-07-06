# Food City — Implementation Plan

> Phased plan to deliver the Food City platform. Each phase produces a
> shippable increment. Estimated effort and session mapping included.

---

## Phase 0: Foundation (this session)

**Goal:** Research, plan, scaffold repo, working auth slice, Docker dev env.

| # | Task | Status |
|---|---|---|
| 0.1 | Edge cases research doc | ✅ done |
| 0.2 | Workflows doc | ✅ done |
| 0.3 | Architecture doc | ✅ done |
| 0.4 | API contract doc | ✅ done |
| 0.5 | Cargo.toml + dependencies | ✅ done |
| 0.6 | Backend src/ structure (modules, error, config) | ✅ done |
| 0.7 | Frontend package.json + Vite + Tailwind | ✅ done |
| 0.8 | Docker Compose (PG + Redis + backend + frontend) | ✅ done |
| 0.9 | Migrations: users, restaurants (minimal) | ✅ done |
| 0.10 | Auth slice: register + login + JWT middleware (backend) | ✅ done |
| 0.11 | Auth slice: login + register pages + auth store (frontend) | ✅ done |
| 0.12 | README with run instructions | ✅ done |

**Deliverable:** Repository with both apps runnable via `docker compose up`,
auth working end-to-end, ready for vertical feature buildout.

---

## Phase 1: Customer Discovery MVP

**Goal:** Customer can browse restaurants, view menus, build a cart.

| # | Task | Est |
|---|---|---|
| 1.1 | Migration: restaurants, menu_versions, menu_items, customizations | 2h |
| 1.2 | Restaurant repo + service (CRUD, search by geo, filter by cuisine) | 3h |
| 1.3 | Menu service (versioning, scheduled publish stub) | 2h |
| 1.4 | Seed data: 10 sample restaurants with full menus | 1h |
| 1.5 | API: `GET /restaurants`, `GET /restaurants/:id`, `GET /restaurants/:id/menu` | 2h |
| 1.6 | Frontend: HomePage (restaurant list, search, filters) | 3h |
| 1.7 | Frontend: RestaurantPage (menu, categories, item cards) | 3h |
| 1.8 | Frontend: Cart drawer (add/remove, quantity, customizations) | 3h |
| 1.9 | Cache layer for restaurant list + menu (Redis, 60s TTL) | 2h |

**Exit criteria:** Customer can search, browse, build a cart (not yet
checkout).

---

## Phase 2: Order Placement & Payments

**Goal:** Customer can checkout, pay, and receive order confirmation.

| # | Task | Est |
|---|---|---|
| 2.1 | Migration: carts, cart_items, orders, order_items, order_events | 2h |
| 2.2 | Cart service (snapshot to order, lock cart) | 3h |
| 2.3 | Order service (create, state machine, inventory atomic deduct) | 4h |
| 2.4 | Payment service (Stripe PaymentIntent, idempotency) | 4h |
| 2.5 | Stripe webhook handler + idempotent event log | 3h |
| 2.6 | API: `POST /cart`, `POST /orders`, `GET /orders/:id` | 3h |
| 2.7 | Promo code validation (atomic counter, per-user cap) | 2h |
| 2.8 | Frontend: CheckoutPage (address, payment, promo, tip) | 4h |
| 2.9 | Frontend: OrderConfirmationPage | 1h |
| 2.10 | Frontend: OrdersPage (history) | 2h |
| 2.11 | E2E test: full checkout flow with Stripe test cards | 2h |

**Exit criteria:** Customer can complete a paid order end-to-end.

---

## Phase 3: Realtime Order Tracking

**Goal:** Customer sees live order status + driver location on map.

| # | Task | Est |
|---|---|---|
| 3.1 | WS gateway in Axum (auth, subscribe, fan-out) | 4h |
| 3.2 | `order_events` table + event publishing in order state changes | 2h |
| 3.3 | Reconnect with `last_event_id` replay | 2h |
| 3.4 | Driver service: location updates (Redis GEO), smoothing | 3h |
| 3.5 | Driver matching background job (broadcast + accept) | 4h |
| 3.6 | Frontend: WS client with auto-reconnect | 2h |
| 3.7 | Frontend: OrderTrackingPage (status timeline + map) | 4h |
| 3.8 | Frontend: Leaflet map with driver marker | 3h |
| 3.9 | Order acceptance timeout worker | 2h |
| 3.10 | Driver pickup watchdog worker | 2h |

**Exit criteria:** Order placement → acceptance → (simulated) driver
delivery, customer sees live updates.

---

## Phase 4: Restaurant Dashboard

**Goal:** Restaurant can receive orders, manage menu, respond to reviews.

| # | Task | Est |
|---|---|---|
| 4.1 | Restaurant onboarding flow (KYC stub, admin approval) | 4h |
| 4.2 | Restaurant dashboard layout + auth guard | 2h |
| 4.3 | Live order queue with WS subscription | 3h |
| 4.4 | Accept/reject order UI + API | 2h |
| 4.5 | Mark order preparing / ready | 2h |
| 4.6 | Menu management UI (CRUD items, categories, customizations) | 6h |
| 4.7 | Hours management (per-day + exceptions) | 2h |
| 4.8 | Reviews list + reply UI | 3h |
| 4.9 | Earnings page (daily/weekly summary) | 3h |
| 4.10 | Restaurant status toggle (active/paused/closing) | 1h |

**Exit criteria:** Restaurant can fully operate via dashboard.

---

## Phase 5: Driver App

**Goal:** Driver can go online, receive offers, deliver orders.

| # | Task | Est |
|---|---|---|
| 5.1 | Migration: drivers, delivery_proofs, driver_location_history | 1h |
| 5.2 | Driver onboarding (license, vehicle, Stripe Connect) | 3h |
| 5.3 | Shift toggle + heartbeat + location push | 3h |
| 5.4 | Order offer UI (15s accept window) | 3h |
| 5.5 | Active delivery flow (pickup → deliver → proof) | 4h |
| 5.6 | Driver earnings dashboard | 2h |
| 5.7 | Ratings + acceptance rate display | 1h |

**Exit criteria:** Driver can complete a delivery end-to-end.

---

## Phase 6: Reviews, Loyalty, Disputes

| # | Task | Est |
|---|---|---|
| 6.1 | Reviews: post, list, reply, photo upload | 4h |
| 6.2 | Review eligibility enforcement (order exists, not already reviewed) | 1h |
| 6.3 | Anomaly detection job (review bombing) | 2h |
| 6.4 | Loyalty: points accrual on delivery | 2h |
| 6.5 | Loyalty: tier computation (nightly job) | 2h |
| 6.6 | Loyalty: points redemption at checkout | 3h |
| 6.7 | Disputes: customer flow + admin queue | 4h |
| 6.8 | Auto-refund rules for missing items | 2h |

---

## Phase 7: Admin Console

| # | Task | Est |
|---|---|---|
| 7.1 | Live ops dashboard (active orders, drivers, restaurants) | 4h |
| 7.2 | Verification queue UI | 2h |
| 7.3 | Dispute resolution UI | 3h |
| 7.4 | Promo code management UI | 3h |
| 7.5 | Analytics dashboards (GMV, retention, etc.) | 4h |

---

## Phase 8: Hardening & Scale

| # | Task | Est |
|---|---|---|
| 8.1 | Read replica setup + read/write routing | 3h |
| 8.2 | Cache singleflight + stale-while-revalidate | 2h |
| 8.3 | Rate limiting (per-IP + per-user) | 2h |
| 8.4 | Circuit breaker on order placement (queue depth) | 2h |
| 8.5 | Prometheus metrics + Grafana dashboards | 3h |
| 8.6 | Load testing (k6) — simulate lunch rush | 4h |
| 8.7 | DB index audit + slow query log review | 2h |
| 8.8 | Zero-downtime migration review checklist | 1h |
| 8.9 | Fraud detection jobs (promo abuse, ATO signals) | 4h |

---

## Phase 9: Production Readiness

| # | Task | Est |
|---|---|---|
| 9.1 | CI/CD pipeline (GitHub Actions) | 3h |
| 9.2 | Docker production images (multi-stage, distroless) | 2h |
| 9.3 | Kubernetes manifests (or Fly.io / Railway config) | 3h |
| 9.4 | Secret management (SOPS / Vault) | 2h |
| 9.5 | Backup + restore runbook (PG, Redis) | 2h |
| 9.6 | Runbooks for common incidents | 3h |
| 9.7 | SSL/TLS termination, CDN for frontend | 2h |
| 9.8 | Stripe Connect onboarding flow for production | 2h |

---

## Session Mapping (this multi-session project)

- **Session 1 (now):** Phase 0 — done at end of this session.
- **Session 2:** Phases 1 + 2 (customer MVP: discovery → checkout → pay).
- **Session 3:** Phase 3 (realtime tracking + driver matching).
- **Session 4:** Phases 4 + 5 (restaurant + driver apps).
- **Session 5:** Phases 6 + 7 (reviews, loyalty, disputes, admin).
- **Session 6:** Phases 8 + 9 (hardening + production).

---

## Definition of Done (per phase)

- All tasks in phase completed and merged to `main`.
- Unit tests for service layer (>70% coverage on new code).
- Integration test for the user-facing flow.
- Documentation updated (API_CONTRACT.md, README.md).
- No `clippy` warnings, no `tsc` errors.
- Manual smoke test passes against Docker Compose stack.


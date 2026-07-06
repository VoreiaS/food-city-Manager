# Food City — Self-Audit Report

> Performed by the dev team after the 9-phase build + 4 polish iterations.
> Goal: identify what's lagging, missing, or broken, and prioritize fixes.

---

## Summary

| Area | Score | Notes |
|---|---|---|
| Backend correctness | 7/10 | 2 background workers are still stubs; promo codes table exists but no service; menu versioning not implemented |
| Backend hygiene | 6/10 | 11 cargo warnings (unused imports, dead code, stub comments) |
| Frontend completeness | 7/10 | 7 of 19 pages are stubs; profile, reviews, earnings, analytics, verifications, driver offers not built |
| Frontend UX | 6/10 | No error boundary, no 404 page, no mobile nav, sparse loading skeletons, no image fallbacks |
| Security | 7/10 | Auth guards are role-only (no ownership checks on order/driver endpoints); no input sanitization on review bodies |
| Tests | 2/10 | Zero backend tests, zero frontend tests — only `cargo check` and `tsc` in CI |
| Ops | 7/10 | CI, Docker, runbook all exist; no K8s manifest, no healthcheck script in compose, no metrics dashboard |
| Docs | 8/10 | 5 doc files are excellent; README is current; missing CONTRIBUTING and API examples |

---

## Critical Findings (P0 — must fix)

### 1. Two background workers are still no-op stubs
- `workers/order_acceptance_timeout.rs` — supposed to auto-reject orders restaurants don't accept in 5 min. Currently just sleeps. **Customer gets stuck waiting forever if restaurant is offline.**
- `workers/payment_reconciler.rs` — supposed to poll Stripe for missed webhooks. Currently just sleeps. **Orders can get stuck in `pending` payment status indefinitely if webhook is missed.**

### 2. Promo codes have a table but no service
- `promo_codes` + `promo_redemptions` tables exist in migration `20260101000002`
- `CreateOrderRequest` accepts `promo_code: Option<String>` but `order_service::place_order` hardcodes `discount_cents = 0`
- No API to list/create/validate promo codes
- Admin can't manage promos

### 3. Menu versioning not implemented
- `menu_versions` table exists with `version` + `is_active` columns
- But `restaurant_dashboard::create_item` and `update_item` directly mutate `menu_items` without creating a new version
- Edge case from `EDGE_CASES.md` (EC-5.5): price change between cart and order is not properly handled — currently the cart snapshot uses current price at order time, which is correct, but there's no audit trail of price changes

### 4. Customer ProfilePage is a stub
- `/profile` route exists but shows "Not Implemented"
- Users can't edit name, phone, password, or manage addresses from a central place

### 5. Restaurant ReviewsPage is a stub
- `/restaurant/reviews` shows "Not Implemented"
- Backend `reviews.rs` already has the reply endpoint — just no UI

### 6. Restaurant EarningsPage is a stub
- `/restaurant/earnings` shows "Not Implemented"
- Backend has `payout_ledger` table but no earnings API

### 7. Driver EarningsPage + OrderOfferPage are stubs
- Driver can't see earnings history
- Driver can't see/respond to incoming order offers (only auto-assign works)

### 8. Admin VerificationsPage + AnalyticsPage are stubs
- `/admin/verifications` shows "Not Implemented" — restaurant KYC queue missing
- `/admin/analytics` shows "Not Implemented" — but `/admin/analytics/summary` API exists; just no charts UI

---

## High-Severity Findings (P1)

### 9. No error boundary on frontend
- A single React render error crashes the entire app to white screen
- Should wrap the router in `<ErrorBoundary>` with a fallback UI

### 10. No 404 / catch-all route
- Unknown URLs fall through to a blank page
- Need `<Route path="*" element={<NotFoundPage />} />`

### 11. No mobile navigation
- Header nav links are `hidden md:flex` — on mobile, users can't reach Orders/Dashboard/etc.
- Need a hamburger menu or bottom nav for small screens

### 12. No image fallbacks
- Restaurant cards/logos that fail to load show broken-image icon
- Menu item photos missing → ugly empty box
- Need `onError` handlers with placeholder

### 13. Sparse loading skeletons
- Only `HomePage` has skeleton loaders (8 pulsing cards)
- Restaurant detail, orders list, menu editor all show plain "Loading…" text
- Should have skeleton matching the final layout

### 14. Auth token expiry not checked client-side
- `authStore` stores `expiresAt` but never checks it
- User with expired token sees 401 → auto-refresh, but the initial request still fires
- Should proactively refresh if token expires in < 30s

### 15. Backend: 11 cargo warnings
- Unused imports (`post`, `IntoResponse`, `StatusCode`, `header`, `Duration`, `PgPool`)
- Unused variables (`order`, `version`, `e`)
- Dead code (`ServerMsg::DriverLocation`, `ServerMsg::Snapshot` variants never constructed)
- `jwt.rs` `secret` field never read
- These should be cleaned up — CI's `clippy -D warnings` would fail

---

## Medium-Severity Findings (P2)

### 16. Zero tests
- No `backend/tests/` directory
- No `*.test.ts` files in frontend
- CI runs `cargo test --release` and `npm run typecheck` but there's nothing to run
- Should add at least: auth service tests, order state machine tests, cart service tests, frontend component smoke tests

### 17. No ownership checks on key endpoints
- `GET /orders/:id` — any authenticated user can fetch any order (not just their own)
- `POST /orders/:id/cancel` — same
- `POST /drivers/orders/:id/accept` — any driver can accept any order (not just the assigned one)
- Should verify `order.customer_id == auth.user_id` for customer endpoints

### 18. No K8s manifest
- `docker-compose.prod.yml` exists but no Kubernetes YAML
- Runbook mentions K8s but provides no manifests
- Should add at least a deployment + service + ingress template

### 19. No healthcheck in docker-compose for backend/frontend
- Postgres + Redis have healthchecks
- Backend + frontend containers don't — Docker can't tell if they're actually serving

### 20. No metrics dashboard
- `/metrics` endpoint exists (Prometheus format)
- No Grafana dashboard JSON to visualize them

### 21. Restaurant onboarding (KYC) not implemented
- `restaurant_verifications` table exists
- But no API for restaurant owner to submit docs, no admin UI to approve
- Currently restaurants go straight to `active` status on seed data

### 22. Driver offer broadcast not implemented
- `driver_match_loop` auto-assigns nearest driver
- Real workflow (per `WORKFLOWS.md` WF-D2): broadcast to top 5 drivers, first accept wins
- Currently drivers have no way to see incoming offers

---

## Low-Severity Findings (P3)

### 23. No CONTRIBUTING.md
- README mentions "see CONTRIBUTING" but file doesn't exist

### 24. No API examples in docs
- `API_CONTRACT.md` has the spec but no curl examples

### 25. No Docker healthcheck for backend/frontend in dev compose

### 26. `cache_service::invalidate_prefix` has a placeholder implementation
- Uses `KEYS` which is O(N) and blocks Redis — should use `SCAN`

### 27. Review body not sanitized
- Customer can submit HTML/JS in review body; rendered as text by React (safe) but should still strip on server

### 28. No rate limit on WS connections
- A single IP could open thousands of WS connections

---

## Fix Plan

### Batch 1 — Critical correctness (P0)
1. Implement `order_acceptance_timeout` worker — auto-reject after 5 min + refund
2. Implement `payment_reconciler` worker — poll Stripe for pending intents
3. Build promo code service + API (validate, redeem, admin CRUD)
4. Wire promo codes into `order_service::place_order`
5. Build customer `ProfilePage` (edit name/phone, manage addresses)
6. Build restaurant `ReviewsPage` (list + reply)
7. Build restaurant `EarningsPage` (daily/weekly summary from payout_ledger)
8. Build driver `EarningsPage` + `OrderOfferPage`
9. Build admin `VerificationsPage` + `AnalyticsPage`

### Batch 2 — UX polish (P1)
10. Add `<ErrorBoundary>` wrapping the router
11. Add `<NotFoundPage />` + catch-all route
12. Add mobile hamburger menu to Header
13. Add image `onError` fallbacks (restaurant logos, menu photos)
14. Add loading skeletons to RestaurantDetail, OrdersPage, MenuPage
15. Proactive token refresh in authStore (check expiry before request)

### Batch 3 — Backend hygiene (P1)
16. Fix all 11 cargo warnings (remove unused imports/vars/dead code)
17. Add ownership checks to customer order endpoints
18. Fix `cache_service::invalidate_prefix` to use SCAN
19. Add rate limit on WS connection per IP

### Batch 4 — Tests + ops (P2)
20. Add backend integration tests (auth, order state machine, cart)
21. Add frontend smoke tests (render each page without crash)
22. Add K8s manifest (deployment + service + ingress)
23. Add healthchecks to docker-compose for backend/frontend
24. Add CONTRIBUTING.md
25. Add Grafana dashboard JSON

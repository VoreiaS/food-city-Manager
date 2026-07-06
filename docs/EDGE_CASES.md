# Food City — Edge Cases & Failure Modes Research

> Research document covering all 8 edge-case domains the team flagged as
> critical for a Zomato/Swiggy-style restaurant marketplace with full
> delivery: **Concurrency, Geo+Distance, Payments, Realtime, Orders,
> Fraud, Multi-tenant, Scale**.
>
> For each domain we list: real-world failure scenarios → root cause →
> mitigation strategy → implementation note (where in the codebase it
> lives).

---

## 1. Concurrency & Race Conditions

### 1.1 Two customers ordering the last menu item simultaneously
**Scenario:** A restaurant has 1 unit of "Chef's Special Biryani" left
(inventory-tracked item). Two customers hit "Place Order" within the
same 200ms window. Both pass the availability check, both payments
succeed, but only one meal exists.

**Root cause:** Read-then-write pattern without a lock. Naive code does
`SELECT stock FROM menu_items WHERE id = $1` then `UPDATE ... SET stock
= stock - 1`. Two transactions interleave.

**Mitigation:**
- **Atomic UPDATE with check:** `UPDATE menu_items SET stock = stock - 1
  WHERE id = $1 AND stock >= $qty RETURNING stock;` — if 0 rows
  returned, the item is sold out. This is a single atomic statement
  using PostgreSQL row-level lock.
- **`SELECT ... FOR UPDATE` pessimistic lock** when the logic is more
  complex (e.g., bundled items).
- **SERIALIZABLE isolation** only for the narrow slice that needs it;
  don't apply globally (perf cost).
- **Outbox pattern**: write the order + outbox event in the same
  transaction; a separate worker deducts inventory asynchronously with
  retries. If inventory fails, mark order as `inventory_failed` and
  refund.

**Implementation:** `backend/src/services/order_service.rs` —
`reserve_inventory()` uses `UPDATE ... RETURNING`. Inventory is
optional per item (`track_stock` flag); most restaurants in marketplace
mode don't track stock per dish, but those that do get atomic
deduction.

### 1.2 Double-payment on retry / network blip
**Scenario:** Customer clicks "Pay", the request times out at the
gateway, frontend retries. Customer charged twice.

**Mitigation:**
- **Idempotency keys**: every payment intent carries a
  client-generated `idempotency_key` (UUID). Backend stores
  `payment_intents(idempotency_key UNIQUE, status, ...)`. On retry,
  return the existing intent instead of creating a new one.
- Stripe itself supports idempotency keys at the API layer; we mirror
  that at our layer too so non-Stripe gateways work.
- Payment intent states: `pending → requires_capture → succeeded |
  failed | canceled`. Transitions are guarded by `UPDATE ... WHERE
  status = 'pending'` (optimistic lock).

**Implementation:** `backend/src/services/payment_service.rs` +
`payment_intents` table with `idempotency_key TEXT UNIQUE`.

### 1.3 Cart modified during checkout
**Scenario:** Customer has items in cart, opens checkout, in another
tab adds/removes items, then submits payment. The paid cart doesn't
match current cart.

**Mitigation:**
- Checkout **snapshots** the cart into an `order` row with
  `order_items` JSONB snapshot of name/price/customizations at that
  moment. Price is locked.
- After payment success, the original cart is **frozen** (status =
  `locked`) until order completes or fails. If failed, cart unfreezes.
- Frontend: disable cart edits while on checkout step.

**Implementation:** `orders.snapshot_cart()` in `order_service.rs`;
`carts.status` enum: `active | locked | converted | abandoned`.

### 1.4 Promo code abuse via concurrent redemption
**Scenario:** Promo code `WELCOME50` has a global cap of 1000 uses. At
999 uses, 5 customers submit simultaneously — all 5 succeed, cap
exceeded.

**Mitigation:**
- Atomic counter: `UPDATE promo_codes SET used_count = used_count + 1
  WHERE code = $1 AND used_count < max_uses RETURNING *;` — 0 rows =
  exhausted.
- Per-user cap: composite unique `(promo_code_id, user_id)` on
  `promo_redemptions` table.
- Time-window cap: track `used_count` per day if `daily_cap` is set.

**Implementation:** `promo_codes` table + `promo_redemptions` table.

### 1.5 Driver accepting multiple orders simultaneously
**Scenario:** Driver gets two nearby order assignments, accepts both,
but the system only allows one active order per driver.

**Mitigation:**
- Driver state machine: `offline | available | assigned | en_route |
  at_restaurant | picked_up | delivering | delivered`. Only
  `available` drivers can accept; accepting transitions to `assigned`
  atomically.
- `UPDATE drivers SET status = 'assigned', current_order_id = $1 WHERE
  id = $2 AND status = 'available' RETURNING *;` — 0 rows = already
  took another order.

**Implementation:** `drivers` table state machine; Redis sorted set
for available-driver pool.

---

## 2. Geo & Distance

### 2.1 Customer outside service area places order
**Scenario:** Customer's address geocodes to a point 30km from the
nearest restaurant, but they somehow add items and reach checkout.

**Mitigation:**
- **Service area polygon**: each restaurant has a `service_polygon
  GEOGRAPHY(POLYGON)` or simpler `delivery_radius_m INTEGER`. At
  checkout, backend re-validates: `ST_DWithin(restaurant.geom,
  customer.geom, radius)`.
- Frontend: warn at restaurant page if address is outside range —
  disable "Add to Cart".
- Geofence violation mid-flow (customer changes address on checkout):
  re-validate on every address change.

**Implementation:** `geo_service.rs` uses `ST_DWithin` and
`ST_MakePoint`; restaurants expose `delivers_to(lat, lng)`.

### 2.2 Restaurant's delivery radius overlaps but driver pool is empty
**Scenario:** Customer is in range, restaurant is open, but no driver
is available within 5km. Order accepted, then waits 30min, customer
cancels.

**Mitigation:**
- **Driver availability heatmap**: Redis GEO set
  `drivers:available` containing driver locations. At order acceptance,
  estimate driver ETA via `GEORADIUS BYRADIUS`. If no driver within
  `max_driver_distance_km` (configurable per city), warn restaurant
  before accepting: "High delivery delay expected (25+ min)".
- **Soft acceptance**: restaurant can decline if ETA > threshold.
- **Driver reassignment loop**: every 30s, if no driver accepted,
  expand search radius by 1km up to a cap.

**Implementation:** `realtime_service.rs` + Redis GEO commands.

### 2.3 Address geocoding ambiguity
**Scenario:** Customer types "Main St" — exists in 3 cities. Geocoder
returns the wrong one. Order routed to wrong city.

**Mitigation:**
- Always geocode with city/region context (bias to user's last known
  city).
- Store both `lat/lng` and `formatted_address` and display the latter
  to user for confirmation before save.
- Reverse-geocode the saved point to confirm it matches user input.

**Implementation:** Use Nominatim/Mapbox; store full payload, show
`formatted_address` in UI.

### 2.4 Driver GPS jitter / inaccurate pin
**Scenario:** Driver's phone GPS bounces 50m while stationary.
Customer sees driver "moving" on map, ETA fluctuates wildly.

**Mitigation:**
- **Server-side smoothing**: keep last N=10 locations, compute median
  or use Kalman filter. Reject updates > 200km/h (impossible on roads).
- **ETA recalc throttling**: only recompute ETA every 30s, not on
  every location push.
- **Tolerance band**: if new point is < 10m from last, ignore.

**Implementation:** `driver_location_buffer` Redis stream; smoothing
in `realtime_service.rs`.

### 2.5 Restaurant relocates / closes permanently mid-order
**Scenario:** Customer orders, restaurant marks itself closed
permanently 2 minutes later. Driver arrives to locked doors.

**Mitigation:**
- Restaurant close action requires all in-flight orders to be
  `delivered` or `canceled` first. Soft-close: stop accepting new
  orders, finish existing.
- Admin can force-close with bulk-cancel + auto-refund workflow.

**Implementation:** `restaurants.status` enum:
`active | paused | closing | closed`. "closing" rejects new orders
but allows in-flight completion.

---

## 3. Payments

### 3.1 Webhook arrives before HTTP response
**Scenario:** Customer pays via Stripe. Stripe sends `payment_intent.succeeded`
webhook while our `/pay` endpoint is still waiting for the sync
response. Webhook handler tries to update order; race against the
sync path.

**Mitigation:**
- **Idempotent webhook handler**: webhook events table with
  `stripe_event_id UNIQUE`. If we already processed this event, return
  200.
- **State machine guards**: `UPDATE orders SET payment_status =
  'paid' WHERE id = $1 AND payment_status = 'pending' RETURNING *;` —
  0 rows means already processed (either by sync or webhook), still
  return 200 to Stripe.
- **Never trust client-reported status**: only server-side webhooks
  and server-side sync responses drive state.

**Implementation:** `payment_webhooks` table; `payment_service.rs`
guards all transitions with `WHERE` clauses.

### 3.2 Refund for canceled order with already-captured payment
**Scenario:** Customer cancels after driver pickup. Payment is
captured. Refund needed. But partial: restaurant already prepared
food, may charge a fee.

**Mitigation:**
- **Cancellation policy windows**:
  - Before restaurant accepts: full refund.
  - After accept, before prepare starts: full refund + restaurant
    penalty configurable.
  - During preparation: 50% refund (restaurant fee).
  - After pickup: no refund (restaurant fulfilled, driver en route).
- Refund amount = order_total − cancellation_fee.
- Stripe partial refund API supports amount parameter.

**Implementation:** `order_service.rs::compute_cancellation_refund(order,
cancel_time)`; admin-configurable policy per restaurant.

### 3.3 Split payouts to restaurants + platform fee
**Scenario:** Order is $50. Platform takes 15% commission = $7.50.
Restaurant gets $42.50. Driver gets $3 delivery fee. Need to
transfer $42.50 to restaurant's Stripe connected account.

**Mitigation:**
- **Stripe Connect**: each restaurant onboards as a connected account.
  At order completion, create a Transfer of $42.50 to their account.
- **Ledger table**: `payout_ledger` records every movement:
  `order_id, payee, amount, type (platform_fee | restaurant_payout |
  driver_payout), stripe_transfer_id, status`.
- **Reconciliation job**: nightly cron compares ledger vs Stripe
  transfers, alerts on mismatches.
- **Driver payout**: aggregated weekly (or per-order for instant
  payout) to driver's connected account or bank.

**Implementation:** `payout_service.rs`; Stripe Connect Onboarding
flow for restaurants/drivers.

### 3.4 Failed webhook retries / silent failure
**Scenario:** Stripe retries a webhook 3 times, all fail because our
DB is briefly down. Stripe gives up. Order stuck in `pending`.

**Mitigation:**
- **Stripe dashboard manual replay**: but better, **polling
  reconciler**: every 5 min, fetch recent payment intents from Stripe
  API, compare to our DB, heal discrepancies.
- Webhook endpoint must return 2xx **only** after DB commit; on DB
  error return 5xx so Stripe retries.

**Implementation:** `payment_reconciler` background task in
`main.rs` using `tokio::spawn` + `tokio::time::interval`.

### 3.5 Chargeback / dispute handling
**Scenario:** Customer disputes a charge with their bank 30 days
later. Stripe sends `charge.dispute.created`. Funds pulled back from
restaurant.

**Mitigation:**
- Mark order `disputed`, freeze driver/restaurant payouts for that
  order.
- Surface to admin dashboard with evidence submission workflow.
- If dispute lost, claw back from restaurant ledger; if their balance
  is insufficient, mark as negative and recover from future payouts.

**Implementation:** `disputes` table; admin notification channel.

---

## 4. Realtime (WebSocket)

### 4.1 Customer's WebSocket drops mid-delivery
**Scenario:** Customer on subway, connection drops for 3 minutes.
Driver delivers order. Customer reconnects, sees stale "out for
delivery" status.

**Mitigation:**
- **Missed-event sync**: on reconnect, client sends
  `last_event_id` (monotonic per-order event ID). Server replays all
  events with `id > last_event_id` for that order.
- **Event log table**: `order_events(id BIGSERIAL, order_id, event_type,
  payload JSONB, created_at)`. WebSocket subscriber replays from here.
- **Snapshot fallback**: if client's `last_event_id` is too old (>1h),
  send current full order state instead of replay.

**Implementation:** `order_events` table; `ws.rs::handle_reconnect`.

### 4.2 Driver goes offline mid-delivery
**Scenario:** Driver's phone battery dies 5 min after pickup. Customer
sees stale location. Order stuck.

**Mitigation:**
- **Heartbeat timeout**: driver must ping every 15s. After 60s of
  silence, mark driver `unreachable`, alert admin.
- **Reassignment policy**: if driver doesn't recover in 5 min, admin
  can reassign order to a new driver. Old driver's payout prorated.
- **Customer notification**: "Driver may be experiencing connection
  issues. We're checking in."

**Implementation:** `driver_heartbeats` Redis key with TTL 60s;
background reaper task.

### 4.3 WebSocket fan-out thundering herd
**Scenario:** Dinner rush — 10,000 concurrent WebSocket connections,
each subscribing to 1-2 orders. Every order status change fans out to
subscribers. Burst of 500 orders updating simultaneously saturates
the event loop.

**Mitigation:**
- **Redis pub/sub channels per order**: `order:{id}:events`. WebSocket
  workers subscribe only to channels for their connected clients.
- **Batching**: coalesce events within 100ms windows into a single
  message.
- **Backpressure**: if a client's send buffer > 100 messages, drop
  location-only updates (keep state transitions), warn client.
- **Horizontal scale**: multiple WS workers behind a sticky-session
  load balancer; Redis pub/sub syncs across workers.

**Implementation:** `realtime_service.rs` uses
`redis::aio::PubSub`; workers can scale independently of REST API.

### 4.4 Order status events arriving out of order
**Scenario:** Network delay causes `preparing` event to arrive after
`ready` event at client. UI shows wrong state.

**Mitigation:**
- Each event carries `sequence` (per-order monotonic) and
  `order_status` (current snapshot). Client ignores events with
  `sequence < last_seen_sequence` and trusts the snapshot field.
- State machine on client side only allows forward transitions;
  backward transitions require explicit admin override event.

**Implementation:** `order_events.sequence` per-order sequence
counter via `LAG()` window or per-order atomic counter in Redis.

### 4.5 Stale driver location shown after delivery complete
**Scenario:** Order delivered. Driver stays on the map for the next
customer's view because old location wasn't cleared.

**Mitigation:**
- On `delivered` event, driver's `current_order_id` is cleared and
  they go back to `available` pool. Their location is no longer
  published to that customer's channel.
- Customer's map UI unmounts driver marker on `delivered`.

**Implementation:** `realtime_service.rs::publish_delivery_complete`
clears driver-to-order binding.

---

## 5. Orders

### 5.1 Restaurant closed mid-order acceptance
**Scenario:** Customer places order at 11:59 PM. Restaurant hours end
at midnight. At 12:01 AM, restaurant doesn't see the order because
their dashboard auto-logged-out due to closing hours.

**Mitigation:**
- **Order placement hours validation**: at order time, check
  `restaurant.is_open_at(now)`. Reject if outside hours.
- **Grace window**: orders placed within 5 min of closing are still
  accepted (configurable). Restaurant sees them on dashboard next
  morning with `late` flag.
- **Auto-reject timer**: if restaurant doesn't accept within
  `auto_reject_seconds` (default 5 min), order auto-cancels with full
  refund.

**Implementation:** `restaurants.hours JSONB` (per-day open/close +
exceptions); `order_acceptance_timeout` background task.

### 5.2 Customer cancels after restaurant started preparing
**Scenario:** Customer cancels 3 min after restaurant clicked "Start
Preparing". Food already being made.

**Mitigation:**
- Cancellation windows (see 3.2). After `preparing` state, only
  admin can cancel, with partial refund policy.
- Restaurant must confirm cancellation request if food is in
  preparation; admin can override.

**Implementation:** `orders.cancellation_window_ends_at` computed at
acceptance; enforced in `order_service.rs::cancel_order`.

### 5.3 Driver no-show at restaurant
**Scenario:** Driver assigned, never arrives at restaurant. Food sits
getting cold.

**Mitigation:**
- **Driver pickup timer**: after assignment, driver must reach
  restaurant within `pickup_eta * 1.5` minutes. If exceeded, system
  alerts.
- **Auto-reassign**: if driver doesn't ping pickup within 15 min of
  `ready` state, mark driver `no_show`, reassign to next available
  driver.
- **Restaurant compensation**: small fee paid to restaurant for cold
  food, funded by no-show driver's penalty.

**Implementation:** `driver_pickup_watchdog` background task per
order.

### 5.4 Order delivered to wrong address
**Scenario:** Driver drops at wrong unit. Customer complains.

**Mitigation:**
- **Delivery confirmation**: driver must capture photo + GPS at
  delivery. Photo stored, GPS compared to customer address (within
  50m tolerance).
- **OTP-based handoff**: customer gets 4-digit OTP, driver enters it
  to mark delivered (skippable for contactless delivery).
- **Dispute flow**: customer can flag "didn't receive" within 24h,
  triggers investigation.

**Implementation:** `delivery_proof` table with photo URL, GPS, OTP.

### 5.5 Menu price changed between cart and order
**Scenario:** Restaurant updates biryani price from $12 → $14 while
customer's cart has it at $12. Order placed at old price; restaurant
loses $2.

**Mitigation:**
- Order snapshots prices (see 1.3). Restaurant sees snapshot price at
  acceptance; if they want to reject, they can.
- **Menu versioning**: menus have `version` field; cart stores
  `menu_version_at_add`. On checkout, if version mismatch, show
  "Prices have changed" modal with diff, customer re-confirms.
- Admin-configurable: auto-update cart to new prices vs. block
  checkout.

**Implementation:** `menu_items.version` (incremented on price/availability
change); `cart_items.menu_version_at_add`.

### 5.6 Order item out of stock after order placed
**Scenario:** Restaurant accepts order, then realizes they're out of
an item.

**Mitigation:**
- **Item-level status**: restaurant can mark specific items
  `out_of_stock` post-acceptance. Customer gets notification, can
  choose: refund that item / substitute / cancel whole order.
- Partial refund flow handled by `payment_service.rs::refund_item`.

**Implementation:** `order_items.status`:
`pending | confirmed | out_of_stock | refunded | prepared`.

---

## 6. Fraud & Abuse

### 6.1 Fake restaurant onboarding
**Scenario:** Scammer registers as restaurant, lists fake menu, takes
orders, never delivers. Pockets the money.

**Mitigation:**
- **KYC verification**: restaurant onboarding requires FSSAI/license
  upload + manual admin review before `active` status.
- **Payout hold**: first 3 payouts held 7 days for fraud review.
- **Bank account verification**: micro-deposit verification before
  first payout.
- **Document verification API**: Stripe Connect handles KYC; we add
  our FSSAI verification step.

**Implementation:** `restaurant_verifications` table; admin review
queue.

### 6.2 Bot-driven promo abuse
**Scenario:** Attacker creates 1000 fake accounts, each redeems
`WELCOME50`, places $0.01 orders, drains promo budget.

**Mitigation:**
- **Per-IP rate limit**: max 5 signups per IP per hour.
- **Device fingerprint**: hash of UA + screen + canvas fingerprint;
  max 3 accounts per fingerprint.
- **Phone/email verification**: OTP required before first order.
- **Promo minimum order value**: `WELCOME50` only valid on orders ≥
  $20.
- **Anomaly detection**: batch job flags accounts that only redeem
  promos and never reorder.

**Implementation:** `rate_limit_middleware`; `device_fingerprints`
table; `promo_codes.min_order_value`.

### 6.3 Review bombing / fake reviews
**Scenario:** Competitor pays people to 1-star a restaurant. Or
restaurant owner 5-stars themselves from employee accounts.

**Mitigation:**
- **Review eligibility**: only customers who completed an order at
  that restaurant in the last 30 days can review.
- **One review per order**: `reviews(order_id UNIQUE)` constraint.
- **Anomaly detection**: sudden spike of 1-star reviews from new
  accounts → flag for admin review, hide temporarily.
- **IP/device correlation**: cluster reviews by device fingerprint;
  flag clusters.

**Implementation:** `reviews` table constraints; anomaly detection
job.

### 6.4 Driver colluding with customer for fake deliveries
**Scenario:** Driver marks order delivered without actually
delivering; customer (friend) confirms; both split refund later via
dispute.

**Mitigation:**
- **GPS proof at delivery** (see 5.4).
- **Photo proof** for contactless.
- **Delivery vs customer address distance check** post-delivery.
- **Pattern detection**: drivers with > 15% of deliveries ending in
  disputes get flagged.

**Implementation:** `delivery_proofs` table + anomaly detection.

### 6.5 Chargeback fraud ("friendly fraud")
**Scenario:** Customer orders, receives, then disputes with bank
saying "never received".

**Mitigation:**
- Strong delivery proof (GPS + photo + OTP).
- Auto-submit evidence to Stripe via API on dispute creation.
- Track repeat disputers; block after 3 disputes in 6 months.

**Implementation:** `dispute_evidence` auto-submission in
`payment_service.rs::handle_dispute`.

### 6.6 Account takeover (ATO)
**Scenario:** Attacker phishes customer credentials, orders expensive
items to a new address, drains saved cards.

**Mitigation:**
- **New address verification**: adding a new delivery address requires
  email/SMS confirmation.
- **Behavioral step-up auth**: high-value order (>$100) or new device
  triggers 2FA.
- **Saved card usage**: only show last 4 digits; require CVV re-entry
  for orders > $50.

**Implementation:** `address_verifications` table; risk-score
middleware in `order_service.rs`.

---

## 7. Multi-Tenant (Restaurant Onboarding)

### 7.1 Menu versioning across scheduled changes
**Scenario:** Restaurant wants to launch a Diwali special menu on Nov
1, auto-revert on Nov 8. Doing it manually = human error.

**Mitigation:**
- **Scheduled menu changes**: `menu_schedules(menu_id, effective_from,
  effective_until, items JSONB)`. Cron job applies at scheduled time.
- **Draft menus**: restaurants edit a draft; publish at scheduled
  time.
- **Versioned menu snapshots**: each published menu is immutable;
  editing creates a new version.

**Implementation:** `menu_versions` table; `menu_publish_scheduler`
background task.

### 7.2 Holiday hours / special closures
**Scenario:** Restaurant closes for a religious holiday. Forgets to
update hours. Orders flood in, all auto-reject.

**Mitigation:**
- **Hours exceptions table**: `restaurant_hours_exceptions(restaurant_id,
  date, is_closed, open_time, close_time)`. Checked before
  `is_open_at()`.
- **Bulk holiday import**: admin can declare city-wide holidays;
  pre-fill exceptions for all restaurants.
- **Closure notice period**: closing for >3 days requires 24h notice;
  customers with upcoming orders in that window get notified.

**Implementation:** `restaurant_hours_exceptions` table; integrated
into `is_open_at()`.

### 7.3 Restaurant owner transferring ownership
**Scenario:** Restaurant sold to new owner. Old owner's account
shouldn't have access; new owner inherits reviews, ratings, history.

**Mitigation:**
- **Ownership transfer flow**: admin-initiated, requires new owner's
  email. Old owner loses access immediately; new owner gets invite.
- **Audit log**: all ownership changes recorded.
- **Menu/recipe preservation**: menus, photos, reviews stay with the
  restaurant entity, not the user account.

**Implementation:** `restaurant_owners(restaurant_id, user_id,
role, granted_at, revoked_at)`; transfer = revoke old + grant new.

### 7.4 Chain restaurants with multiple branches
**Scenario:** "Pizza Hut" has 50 branches. Each has different menu
prices, hours, but shares branding.

**Mitigation:**
- **Restaurant groups**: `restaurant_groups(id, name, logo)`.
  `restaurants.group_id` references group.
- **Per-branch menu override**: branches inherit group menu by
  default; can override individual items.
- **Group-level analytics**: admin can view aggregate vs per-branch.

**Implementation:** `restaurant_groups` table; menu inheritance logic
in `menu_service.rs`.

### 7.5 Restaurant paused vs closed (reversible vs permanent)
**Scenario:** Owner clicks "Close" thinking it's temporary; system
hides them from search permanently. Loses ranking.

**Mitigation:**
- **Clear UX**: "Pause for today" vs "Close permanently" buttons.
- **Paused state**: hidden from new customer search but visible to
  existing customers with orders in last 30 days (for reordering).
  Ranking preserved.
- **Closed state**: hidden entirely after 30 days; ranking reset on
  reopen.

**Implementation:** `restaurants.status`:
`active | paused | closing | closed`; search filter logic.

---

## 8. Scale & Peak Load

### 8.1 Lunch rush (12:00–13:30) traffic spike
**Scenario:** 10× normal traffic during lunch. API latency rises from
200ms to 5s. DB connection pool exhausted.

**Mitigation:**
- **DB connection pool sizing**: `sqlx` pool `max_connections = 50`
  per instance, with `acquire_timeout = 5s`. Reject requests that
  can't acquire rather than queue.
- **Read replicas**: listings, menus, reviews read from replica;
  writes go to primary. `sqlx` with two pools.
- **Redis caching layer**: restaurant list cached 30s; menu cached
  60s; invalidation on update.
- **Auto-scaling**: HPA on CPU > 70%; min 3 replicas, max 20.
- **Graceful degradation**: search returns cached results during
  overload; order placement still works (priority path).

**Implementation:** `db/pool.rs` dual-pool setup;
`cache_middleware` for read-heavy endpoints.

### 8.2 Database write contention on hot rows
**Scenario:** A popular restaurant's `view_count` or `order_count`
column gets updated on every page view / order. Row lock
contention.

**Mitigation:**
- **Counter sharding**: `restaurant_counters(restaurant_id, shard,
  count)` with N shards per restaurant. Reads sum across shards.
  Writes go to random shard, reducing contention.
- **Async aggregation**: increment Redis counter, flush to DB every
  1 min via batch UPDATE.
- **Skip locked views**: views are eventually consistent (acceptable
  for display).

**Implementation:** `counter_service.rs` Redis-backed.

### 8.3 Order placement queue backpressure
**Scenario:** 1000 orders/min during peak. Order service can
process 500/min. Without backpressure, queue grows, latency
balloons, customers retry, more load.

**Mitigation:**
- **Token bucket rate limiter** per endpoint, especially
  `/orders POST`.
- **Queue-based processing**: order placement request enqueues to
  Redis stream `order_queue`; worker processes async. Customer gets
  "order received" response immediately; final confirmation within
  30s.
- **Circuit breaker**: if queue depth > 5000, reject new orders with
  "system busy, try again in 5 min" (503). Better than
  accepting and timing out.
- **Priority queues**: VIP/loyalty customers get priority lane.

**Implementation:** `tower::limit::ConcurrencyLimit` per route;
Redis stream + worker pool in `order_worker.rs`.

### 8.4 Migrations locking hot tables during deploy
**Scenario:** Deploy adds a column to `orders` table (millions of
rows). Migration takes `ACCESS EXCLUSIVE` lock; API blocks for 30s.

**Mitigation:**
- **Zero-downtime migration pattern**:
  1. Add nullable column (fast).
  2. Backfill in batches of 1000 (no lock).
  3. Add NOT NULL constraint after backfill.
  4. Application starts using new column.
- Use `sqlx::migrate!` with explicit `BEGIN; ... COMMIT;` per
  step; avoid long transactions.
- **Dangerous operations** (drop column, change type): do in
  maintenance window or via shadow table + rename.

**Implementation:** `backend/migrations/` with numbered files;
reviewer checklist for lock-taking ops.

### 8.5 WebSocket connection scaling
**Scenario:** 50,000 concurrent WS connections. Single Node/Rust
process can't hold them all; OS limits on file descriptors.

**Mitigation:**
- **Per-process connection cap**: ~10,000 WS per Rust async task
  pool. Scale horizontally with multiple WS gateway instances.
- **Sticky sessions** via load balancer (or use shared Redis pub/sub
  so any instance can deliver to any client).
- **Connection draining**: on deploy, stop accepting new WS, wait
  30s for existing to naturally close or migrate.
- **`ulimit -n 65535`** on host.

**Implementation:** WS gateway as separate binary `ws_gateway` (or
feature flag in main binary); shared Redis pub/sub.

### 8.6 Cache stampede on hot restaurant
**Scenario:** "Trending Restaurant" cache expires. 1000 requests
simultaneously miss cache, hit DB.

**Mitigation:**
- **Mutex-based cache fill**: only first miss acquires lock, others
  wait. Use Redis `SETNX` as lock.
- **Probabilistic early expiration**: each client randomly expires
  cache 0–30s before TTL; spreads misses.
- **Stale-while-revalidate**: return stale value while refreshing in
  background.

**Implementation:** `cache_service.rs` with singleflight +
stale-while-revalidate.

### 8.7 Driver location write amplification
**Scenario:** 5000 drivers, each pinging location every 5s =
1000 writes/s to DB. PG can't keep up.

**Mitigation:**
- **Redis-only hot path**: driver locations go to Redis GEO set
  `drivers:locations` (in-memory). Never written to PG during
  active delivery.
- **Cold path**: on delivery complete, write final GPS + path
  summary to PG for analytics.
- **Time-series DB**: for long-term location history, use Timescale
  or InfluxDB, not PG.

**Implementation:** `realtime_service.rs::update_driver_location`
Redis-only; periodic flush for analytics.

---

## Cross-Cutting Concerns Summary

| Concern | Pattern | Where |
|---|---|---|
| Idempotency | UUID keys + unique constraints | payments, orders |
| Atomic state transitions | `UPDATE ... WHERE status = ...` | all state machines |
| Event sourcing for audit | `order_events` table | orders, payments |
| Background reconcilers | `tokio::spawn` + `interval` | payments, drivers |
| Redis pub/sub fan-out | per-order channels | realtime |
| Redis GEO for proximity | `GEORADIUS` | driver matching |
| Cache singleflight | `SETNX` lock | hot reads |
| Rate limiting | `tower::limit` + Redis token bucket | all endpoints |
| Webhook idempotency | event ID unique constraint | payments |
| Soft delete everywhere | `deleted_at TIMESTAMP` | all tables |


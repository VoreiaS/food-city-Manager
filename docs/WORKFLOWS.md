# Food City — Real Workflows per Role

> End-to-end user journeys for each of the 4 roles (Customer, Restaurant,
> Driver, Admin). Each workflow lists the trigger → system steps → side
> effects → edge cases (cross-references EDGE_CASES.md) →
> notifications. Workflows are written so engineers can directly derive
> API endpoints, DB transactions, background jobs, and UI screens.

---

## Role 1: Customer

### WF-C1: Discovery & First Order (cold-start → checkout)

```
[Onboarding]
 1. Lands on homepage (no auth)
 2. Enters delivery address (geocoded, persisted to local storage)
 3. Server returns restaurants within delivery radius, sorted by:
    - Distance (default)
    - Rating (4.0+ filter)
    - ETA (fastest first)
    - Promos (free delivery / discount)
 4. Customer browses restaurant card → restaurant detail page
 5. Sees: photos, menu (categorized), reviews, hours, delivery fee, ETA
 6. Adds items to cart, with customizations (size, extras, notes)

[Checkout]
 7. Cart drawer → review items → "Go to Checkout"
 8. Select delivery address (or add new — requires OTP if new device)
 9. Select payment method (saved card / new card / COD / wallet)
10. Apply promo code (validated: active, in-window, under-cap, min-order)
11. Review tip selector (5% / 10% / 15% / custom)
12. Place Order button — disabled until all required fields filled

[Post-Order]
13. Order placed → server creates order, snapshots cart, calls payment
14. Customer sees "Order Confirmed" screen with ETA
15. WebSocket subscription to `order:{id}:events` for live updates
16. Notifications at each state: accepted → preparing → ready → picked → delivered
17. On delivery: "Rate your order" prompt, photo proof shown
18. Loyalty points credited 24h after delivery (no dispute filed)

[Re-engagement]
19. Post-delivery: review prompt (optional, requires 1+ words)
20. Reorder CTA on homepage "Order again"
21. Loyalty tier shown in profile; tier-up notification
```

**Edge cases:** EC-1.1 (last item), EC-1.3 (cart modified), EC-2.1 (out of area), EC-3.1 (webhook race), EC-5.5 (price change).

**Background jobs:** order_acceptance_timeout (5min), driver_match_loop (30s), delivery_eta_recalc (30s).

**Notifications:** Push (FCM/APNs), Email (order confirm + receipt), SMS (only on driver arrival if opted in).

---

### WF-C2: Order Tracking (real-time)

```
1. Customer opens "Track Order" page (from notification or Orders tab)
2. WebSocket connects → subscribes to `order:{id}:events`
3. Server sends snapshot of current state (status, driver info, ETA)
4. UI renders:
   - Status timeline (5 steps with current highlighted)
   - Map with restaurant pin → driver pin → customer pin
   - ETA countdown
   - Driver info (name, photo, vehicle, rating, call button)
5. Live updates:
   - Driver location every 5s (smoothed, see EC-4.4)
   - Status transitions on event
   - ETA recalculated every 30s
6. On "out for delivery" → "Call driver" / "Chat driver" enabled
7. On "delivered" → unmount driver marker, show "Rate" prompt
8. If WS drops (EC-4.1): reconnect with last_event_id, replay missed
```

**Edge cases:** EC-4.1 (WS drop), EC-4.3 (fan-out), EC-4.4 (GPS jitter), EC-4.5 (stale driver).

**Server events:** `order.status_changed`, `driver.location_updated`, `driver.assigned`, `driver.unassigned`, `eta.updated`, `order.delivered`.

---

### WF-C3: Reorder & Loyalty Redemption

```
1. Customer opens "Orders" tab → past orders list
2. Taps "Reorder" on a past order
3. Server:
   a. Validates menu items still exist and are in stock
   b. Validates prices (if changed, returns diff — customer confirms)
   c. Validates restaurant is open and delivers to current address
   d. Reconstructs cart with same customizations
4. Customer reviews cart → checkout (same as WF-C1)
5. At payment step: option to apply loyalty points (100 pts = $1)
   - Max 50% of order total redeemable via points
   - Points deducted on order success, refunded on cancellation
```

**Edge cases:** EC-5.5 (price change), EC-5.6 (item out of stock post-reorder).

---

### WF-C4: Review & Rating

```
1. Post-delivery, customer gets review prompt (in-app + push after 1h)
2. Taps "Rate" → opens review modal
3. Selects star rating (1-5) per dimension:
   - Food quality
   - Delivery speed
   - Packaging
   - Overall
4. Optional: text review, photo upload (max 5)
5. Submit → server validates (customer has completed order, not already reviewed)
6. Review published immediately (or held for anomaly check if 1-star spike)
7. Restaurant sees review in dashboard, can reply within 7 days
8. Review influences restaurant's aggregate rating (recomputed nightly)
```

**Edge cases:** EC-6.3 (review bombing).

**Constraints:** `reviews(order_id UNIQUE, customer_id, restaurant_id)` — one review per order.

---

### WF-C5: Dispute / Refund Request

```
1. Customer opens past order → "Report an Issue"
2. Selects issue type:
   - Missing items
   - Wrong order
   - Cold food
   - Late delivery
   - Other
3. Provides description + optional photos
4. Submits → creates `disputes` row, status = `open`
5. Auto-resolution rules:
   - Missing items < $10 with clear photo → auto-refund that item
   - Otherwise → routed to admin queue
6. Admin reviews within 24h, decides refund/partial/reject
7. Customer notified of decision; refund processed via Stripe
8. If customer disagrees → escalation to human review
```

**Edge cases:** EC-3.5 (chargeback), EC-6.5 (friendly fraud).

---

## Role 2: Restaurant

### WF-R1: Onboarding & First Menu

```
[Registration]
 1. Restaurant owner visits "Partner with us" page
 2. Submits: business info, FSSAI license, owner KYC, bank details
 3. Server creates `restaurants` row (status=`pending_verification`)
    + `restaurant_verifications` row
 4. Admin reviews docs (1-2 business days SLA)
 5. On approval: status → `active`, owner gets invite to dashboard

[Dashboard Setup]
 6. Owner logs in, completes profile:
    - Photos (cover, logo)
    - Cuisine type, price range
    - Operating hours (per day + exceptions)
    - Delivery radius (or polygon)
    - Service area map preview
 7. Builds menu:
    - Categories (Starters, Mains, Desserts, Drinks)
    - Items per category: name, description, photo, price, customizations
    - Mark items as veg/vegan/halal, spicy level, allergens
    - Mark items as "bestseller", "new", "seasonal"
 8. Preview menu as customer would see it
 9. Publish menu → restaurant goes live
```

**Edge cases:** EC-6.1 (fake restaurant), EC-7.1 (menu versioning), EC-7.2 (holiday hours).

**Admin jobs:** KYC verification, document OCR, FSSAI cross-check.

---

### WF-R2: Daily Operations — Receiving & Preparing Orders

```
[Order Receipt]
 1. Restaurant dashboard connected via WebSocket to `restaurant:{id}:orders`
 2. New order arrives → audio + visual alert ("New order! Accept within 5 min")
 3. Order card shows: items, customizations, customer address, ETA, payment
 4. Restaurant clicks "Accept" or "Reject" (with reason)
 5. If accept → status = `accepted`, prep timer starts
 6. If reject → order auto-cancels, full refund, logged for analytics
 7. If no response in 5 min → auto-reject + refund + restaurant penalty score

[Preparation]
 8. Restaurant marks items as "preparing" (or per-item status)
 9. Updates "ready" time estimate when starting prep
10. When ready → clicks "Ready for Pickup" → status = `ready`
11. System notifies assigned driver (or assigns if not yet)

[Handoff]
12. Driver arrives → restaurant confirms handoff → status = `picked_up`
13. Restaurant can see driver photo + OTP for verification
14. Order moves out of active queue, archived after 24h
```

**Edge cases:** EC-5.1 (closed mid-order), EC-5.2 (cancel during prep), EC-5.6 (out of stock).

**Timers:** acceptance (5min), prep (configurable per item), ready-pickup (15min before driver no-show alert).

---

### WF-R3: Menu Management & Real-time Updates

```
1. Owner opens "Menu" tab in dashboard
2. Edits draft menu (current menu stays live)
3. Changes can be:
   - Price update (creates new menu_version, see EC-5.5)
   - Item availability toggle (instant, no version bump)
   - New item addition (instant or scheduled)
   - Item removal (instant or scheduled)
4. Bulk operations: "Mark all out of stock" (closing time), "Restore all"
5. Scheduled changes: future-dated publish (Diwali menu, holiday hours)
6. Analytics view: top items, slow movers, profit margins
```

**Edge cases:** EC-5.5 (price change race), EC-7.1 (scheduled menus).

---

### WF-R4: Reviews & Customer Service

```
1. Owner sees new reviews in "Reviews" tab
2. Can reply to reviews (publicly) within 7 days
3. Aggregated rating shown with breakdown by dimension
4. Flag reviews for admin review (inappropriate / fake)
5. dispute management: see open disputes, respond with evidence
6. Trends: rating over time, common complaints (NLP on review text)
```

**Edge cases:** EC-6.3 (review bombing).

---

### WF-R5: Payouts & Financials

```
1. Owner opens "Earnings" tab
2. Sees:
   - Daily / weekly / monthly gross (orders × total)
   - Commission deducted (15% default)
   - Net payout
   - Pending payouts (held for KYC / dispute)
   - Next payout date (weekly Monday)
3. Payout history with Stripe transfer IDs
4. Tax documents (1099-K equivalent) downloadable annually
5. Bank account management (verified via micro-deposits)
```

**Edge cases:** EC-3.3 (split payouts), EC-3.5 (chargeback clawback).

---

## Role 3: Driver

### WF-D1: Shift Start & Availability

```
1. Driver opens driver app, logs in (location permission required)
2. Sees dashboard: earnings today, current shift, ratings
3. Toggles "Go Online" → status = `available`
4. App starts pinging location every 5s to `drivers:locations` (Redis GEO)
5. Driver added to available-driver pool in their current city zone
6. Background heartbeat every 15s to `driver_heartbeats` (Redis, TTL 60s)
7. Driver sees: heat map of orders, expected earnings, surge zones
```

**Edge cases:** EC-2.4 (GPS jitter), EC-4.2 (offline mid-shift), EC-8.7 (write amplification).

---

### WF-D2: Order Assignment & Acceptance

```
[Assignment Models]
 - Auto-assign: server picks best driver (nearest, rating, acceptance rate)
 - Broadcast: server sends to top 5 drivers, first accept wins
 - Manual: driver browses open orders, picks one

[Flow]
 1. Driver receives order offer (push + in-app alert)
    - Pickup location, drop-off location, distance, payout, items count
    - 15-second accept window
 2. Driver accepts → atomic state transition (EC-1.5)
 3. Server sends full order details + customer contact info
 4. Driver navigates to restaurant (in-app map or handoff to Google Maps)

[Pickup]
 5. Arrives at restaurant → marks "At Restaurant" → status = `at_restaurant`
 6. Verifies order items against packing slip, enters OTP if required
 7. Marks "Picked Up" → status = `picked_up` → customer notified

[Delivery]
 8. Navigates to customer address
 9. Live location streamed every 5s → customer sees movement
10. Arrives → marks "At Drop-off" → captures photo + GPS proof
11. Hands off (or marks contactless delivery) → enters OTP if required
12. Marks "Delivered" → status = `delivered`
13. Order complete, payout credited to driver balance
14. Driver returns to available pool for next order
```

**Edge cases:** EC-1.5 (multi-accept), EC-2.2 (no driver), EC-4.2 (offline), EC-5.3 (no-show), EC-5.4 (wrong address), EC-6.4 (collusion).

**Server jobs:** driver_match_loop, driver_pickup_watchdog, delivery_eta_recalc.

---

### WF-D3: Multi-Order Batching (advanced, future)

```
1. Driver on `picked_up` for order A, heading north
2. Server identifies order B from a restaurant on driver's route
   - Pickup within 1km of current path
   - Drop-off within 2km of order A's drop-off
   - Adds < 8 min to total delivery time
3. Server offers batch → driver accepts/rejects
4. Driver picks up B en route, delivers A then B
5. Payouts computed per-order (no batch bonus by default)
```

*Deferred to v2 — design data model to support but don't implement in v1.*

---

### WF-D4: Earnings & Payouts

```
1. Driver sees real-time earnings: per-order breakdown (delivery fee +
   distance + tip + surge bonus)
2. Daily / weekly / monthly summaries
3. Payout schedule:
   - Default: weekly bank transfer (Mondays)
   - Instant payout: 1% fee, min $10 balance
4. Tax documents: annual summary for tax filing
5. Ratings & acceptance rate dashboard (affects priority for assignments)
```

---

## Role 4: Admin

### WF-A1: Restaurant Verification Queue

```
1. Admin opens "Verifications" tab
2. Sees pending restaurant applications (sorted by submission date)
3. Reviews: business docs, FSSAI, KYC, bank account
4. Cross-checks FSSAI number against FSSAI database (API or manual)
5. Approves / Rejects (with reason) → triggers email to owner
6. On approval: owner gets dashboard invite, restaurant status = `active`
7. Suspicious applications flagged for senior review
```

**Edge cases:** EC-6.1 (fake restaurant).

---

### WF-A2: Live Operations Dashboard

```
1. Admin opens "Live Ops" tab — city-wide real-time view
2. Sees:
   - Active orders (map + table)
   - Available drivers (count + map)
   - Driver-to-order ratio (alert if < 0.5)
   - Restaurant status (open/closed/paused)
   - Surge zones (high demand areas)
3. Can manually:
   - Reassign order to different driver
   - Mark restaurant as paused (force-close for emergencies)
   - Refund customer (with reason)
   - Contact driver/restaurant/customer (call/chat)
4. Incident management: log incidents, assign to ops team
```

**Edge cases:** EC-2.2 (driver shortage), EC-5.3 (no-show), EC-8.3 (peak load).

---

### WF-A3: Dispute Resolution

```
1. Admin opens "Disputes" queue
2. Each dispute card: order details, customer complaint, evidence (photos)
3. Decision options: full refund / partial refund / reject (with reason)
4. Auto-routed disputes (missing items < $10) pre-resolved; admin reviews
5. Stripe disputes: pull evidence from delivery proof, submit via API
6. Track dispute resolution SLA (24h target)
7. Pattern detection: customers/restaurants with high dispute rates
```

**Edge cases:** EC-3.5 (chargebacks), EC-6.4 (collusion), EC-6.5 (friendly fraud).

---

### WF-A4: Promotions & Loyalty Management

```
1. Admin opens "Promotions" tab
2. Create promo code:
   - Code, discount type (% / flat / free delivery)
   - Validity window, max uses, per-user cap
   - Min order value, applicable restaurants/categories
   - Customer segment (new / all / VIP)
3. Loyalty program config:
   - Points per $ spent
   - Tier thresholds (Silver 1000pts, Gold 5000pts, Platinum 10000pts)
   - Tier benefits (free delivery, priority support, exclusive promos)
4. Campaign analytics: redemption rate, ROI, customer acquisition cost
5. Bulk push notifications to segments
```

**Edge cases:** EC-1.4 (promo abuse), EC-6.2 (bot abuse).

---

### WF-A5: Analytics & Reporting

```
1. Admin opens "Analytics" tab
2. Dashboards:
   - GMV (gross merchandise value) over time
   - Take rate (commission %)
   - Order volume by city/zone
   - Restaurant performance (top 100, bottom 100)
   - Driver utilization
   - Customer retention (D7, D30)
   - CAC, LTV
3. Export to CSV for finance
4. Schedule weekly email reports to leadership
```

---

## Cross-Role Workflows (System-Initiated)

### WF-X1: Order Lifecycle (state machine)

```
              ┌──────────────────────────────────┐
              │                                  │
              ▼                                  │
  [created]──►[pending_accept]──►[accepted]──►[preparing]──►[ready]──►[picked_up]──►[delivering]──►[delivered]
              │                     │                                                                  ▲
              │                     │                                                                  │
              ▼                     ▼                                                                  │
        [auto_rejected]       [canceled]*                                                             │
              │                                                                                     │
              ▼                                                                                     │
        [refunded]─────────────────────────────────────────────────────────────────────────────────►│
                                                                                                    │
                                                                                          [disputed]──►[resolved]
```

`*canceled` can transition from any pre-`delivered` state; refund amount varies by state.

### WF-X2: Payment Lifecycle

```
[cart_locked]──►[payment_intent_created]──►[payment_pending]──►[payment_succeeded]──►[captured]
                                                  │                       │
                                                  ▼                       │
                                          [payment_failed]                │
                                                  │                       │
                                                  ▼                       │
                                          [retry]──►[payment_failed_x3]   │
                                                                       │
                                                  ┌────────────────────┘
                                                  ▼
                                          [refund_requested]──►[refund_pending]──►[refunded]
                                                                       │
                                                                       ▼
                                                                [refund_failed]──►[manual_review]
```

### WF-X3: Driver Matching Loop (background job)

```
For each `pending_accept` order with no driver:
  1. Find available drivers within `initial_radius` (default 3km)
     - Sorted by: distance, rating, acceptance_rate
  2. Send offer to top N=5 drivers (broadcast)
  3. Wait 15s for acceptance
  4. If accepted → assign, break
  5. If all rejected/timeout → expand radius by 1km, repeat
  6. If radius > `max_radius` (15km) → mark `no_driver_available`
     - Notify customer, offer: wait longer / cancel
  7. Run every 30s per pending order
```

### WF-X4: Realtime Event Fan-out

```
1. State change in DB transaction (e.g., order.preparing)
2. Same transaction writes to `order_events` (id, order_id, type, payload, sequence)
3. After commit, publish to Redis channel `order:{id}:events`
4. All WS workers subscribed to that channel receive event
5. Each worker fans out to its connected clients subscribed to that order
6. Clients receive event, update UI, persist `last_event_id`
7. On reconnect, client sends `last_event_id` → server replays from `order_events` table
```

---

## Notification Matrix

| Event | Customer | Restaurant | Driver | Admin |
|---|---|---|---|---|
| Order placed | Push + Email | Push + Audio | — | Dashboard |
| Order accepted | Push | — | — | Dashboard |
| Order preparing | Push (optional) | — | — | — |
| Order ready | — | — | Push | — |
| Driver assigned | Push + In-app | — | — | — |
| Driver picked up | Push + Map update | — | — | — |
| Driver delivering | Map update only | — | — | — |
| Order delivered | Push + Rate prompt | — | — | — |
| Order canceled | Push + Email | Push | Push | Dashboard |
| Refund processed | Push + Email | — | — | — |
| Review submitted | — | Push | — | — |
| Dispute opened | Push + Email | Push | — | Queue |
| Driver offline > 60s | — | — | — | Alert |
| Restaurant offline > 5min during open hours | — | Push + Call | — | Alert |


# Food City — Real-World Workflow Evaluation

> Walked through each user journey step-by-step against real-world constraints
> and the edge cases documented in `EDGE_CASES.md`. Findings below are bugs
> (wrong behavior) or missing safeguards (no protection against abuse/errors).

---

## Customer Journey — Findings

### C1. Cart allows adding items from closed restaurants ❌ BUG
**Scenario:** Customer opens restaurant page at 11:58 PM. Restaurant closes at midnight. At 12:01 AM customer clicks "Add to Cart". Item is added. At checkout, order fails with "restaurant is closed" — but customer wasted time building a cart.

**Root cause:** `cart_service::add_item` validates menu item belongs to restaurant but never checks `restaurant.is_open(now)` or `restaurant.status == Active`.

**Fix:** Check restaurant open + active status before adding to cart. Return helpful error.

### C2. No max quantity per item ❌ ABUSE
**Scenario:** Customer orders 9,999 biryanis. Stock isn't tracked, so it succeeds. Restaurant receives order, can't fulfill, has to cancel.

**Root cause:** `cart_service::add_item` only checks `quantity >= 1`. No upper bound.

**Fix:** Cap at 99 per item. Restaurants with `track_stock` already enforce real stock.

### C3. No max items per cart ❌ ABUSE
**Scenario:** Customer adds 500 different items to cart. API handles it, but checkout takes 30s, DB insert is huge.

**Fix:** Cap at 50 distinct items per cart.

### C4. No max order value ❌ ABUSE
**Scenario:** Customer adds items totaling $50,000. Stripe may reject (some cards have limits), but we should fail fast.

**Fix:** Cap total at $2,000 (configurable). Return validation error.

### C5. Cancel order doesn't verify ownership ❌ SECURITY BUG
**Scenario:** Customer A knows Customer B's order UUID. A calls `POST /orders/{B's order}/cancel`. Order is canceled!

**Root cause:** `order_service::cancel` takes `_customer_id: Uuid` but the underscore means it's **ignored**. The handler passes the user_id but the service never checks `order.customer_id == customer_id`.

**Fix:** Verify `order.customer_id == customer_id` before canceling.

### C6. No duplicate order placement prevention ❌ DOUBLE-CHARGE
**Scenario:** Customer double-clicks "Place Order" rapidly. Two requests hit backend. Both create separate idempotency keys (fresh UUIDs), both create orders, both charge payment.

**Root cause:** `place_order` generates a new idempotency key every call. The payment intent dedupes by key, but the order itself doesn't.

**Fix:** Generate idempotency key from `cart_id + user_id` hash so retries return the same order.

### C7. Price change between cart and order not detected ❌ EC-5.5 NOT HANDLED
**Scenario:** Customer adds biryani at $12. Restaurant updates price to $14. Customer places order. Order silently uses current price ($14). Customer surprised.

**Root cause:** `cart_items` stores `menu_version_at_add` but `order_service::place_order` fetches current `menu_item.price_cents` instead of comparing to cart's snapshot.

**Fix:** Compare current price to `menu_version_at_add`. If menu version changed, return `409` with price diff so frontend can confirm.

### C8. No dispute time limit ❌ ABUSE
**Scenario:** Customer files dispute 6 months after delivery. Restaurant has discarded records.

**Fix:** Block disputes older than 7 days after delivery.

### C9. Address lat/lng not validated ❌ DATA INTEGRITY
**Scenario:** Customer sets lat=999, lng=999. Geofence checks pass (math doesn't error), driver matching breaks.

**Fix:** Validate `-90 <= lat <= 90` and `-180 <= lng <= 180`.

### C10. No password complexity ❌ WEAK SECURITY
**Scenario:** User sets password "12345678". Passes min 8 check.

**Fix:** Require at least 1 letter + 1 digit. Min 8 chars stays.

### C11. No email/phone verification ❌ FAKE ACCOUNTS
**Scenario:** Attacker registers with someone else's email. Can't verify, but account exists.

**Fix:** Add `email_verified` flag. Block first order until verified. (Full implementation needs email service — flag the field now.)

---

## Restaurant Journey — Findings

### R1. Restaurant can accept any order ❌ SECURITY BUG
**Scenario:** Restaurant A calls `POST /restaurant/orders/{Restaurant B's order}/accept`. Order is accepted by A, even though it belongs to B.

**Root cause:** `restaurant_dashboard::accept_order` calls `require_restaurant` (checks role) but never verifies the order belongs to the caller's restaurant.

**Fix:** Verify `order.restaurant_id == caller's restaurant.id` before any state transition.

### R2. Restaurant can mark any order as preparing/ready ❌ SECURITY BUG
Same root cause as R1. All restaurant-side order transitions lack ownership checks.

**Fix:** Add ownership verification to accept/reject/preparing/ready handlers.

### R3. Menu editor doesn't create new version on price change ❌ EC-5.5
**Scenario:** Restaurant changes biryani price from $12 to $14. `update_item` directly mutates the row. No new `menu_versions` entry. Carts in flight have stale price.

**Root cause:** `restaurant_dashboard::update_item` does raw UPDATE without versioning.

**Fix:** When `price_cents` changes, create a new `menu_versions` row and migrate items to the new version. (Simplified: at minimum, bump a `menu_items.version` counter so cart can detect staleness.)

---

## Driver Journey — Findings

### D1. Any driver can mark any order as picked up / delivered ❌ SECURITY BUG
**Scenario:** Driver A calls `POST /drivers/orders/{Driver B's order}/pickup`. Order transitions. Driver B shows up at restaurant, food already gone.

**Root cause:** `driver_service::pickup_order` and `deliver_order` take `_user_id` but ignore it. No check that the driver is assigned to the order.

**Fix:** Verify `order.driver_id == Some(driver.id)` before pickup/deliver.

### D2. No driver location update throttle ❌ ABUSE
**Scenario:** Driver app bug sends 1000 location updates/sec. Redis + DB flooded.

**Fix:** Throttle to 1 update per 3 seconds per driver (Redis token bucket).

### D3. ETA not recomputed when driver assigned ❌ POOR UX
**Scenario:** Order placed with ETA = now + 45min. Driver assigned 10min later, 2km away. ETA stays at original. Customer sees wrong ETA.

**Fix:** On `driver.assigned` event, recompute ETA based on driver distance to restaurant + restaurant-to-customer distance.

### D4. No vehicle type validation ❌ DATA INTEGRITY
**Scenario:** Driver sets vehicle_type = "rocket". Accepted.

**Fix:** Validate against enum: `bike | scooter | car`.

---

## Admin Journey — Findings

### A1. Admin can reassign to any driver (even offline) ❌ PARTIAL BUG
**Scenario:** Admin reassigns order to a driver who went offline 10min ago. `reassign_order` checks `driver.status == Available` but the driver may have just gone offline.

**Fix:** Already checks available status — adequate. But should also send the driver a notification.

### A2. No audit log for admin actions ❌ COMPLIANCE
**Scenario:** Admin cancels an order, refunds customer. No record of who did it or why.

**Fix:** Add `admin_actions` table logging every admin write action.

---

## Cross-Cutting Findings

### X1. No abandoned cart cleanup ❌ DATA LEAK
**Scenario:** Customer adds items, abandons. Cart sits as `active` forever. DB grows.

**Fix:** Nightly job: mark carts `abandoned` if `updated_at < now() - 24h` and status = active.

### X2. No WS subscription limit ❌ ABUSE
**Scenario:** Client subscribes to 10,000 channels. Each poll cycle hits DB 10,000 times.

**Fix:** Cap at 20 subscriptions per WS connection.

### X3. No Stripe min amount check ❌ PAYMENT FAILURE
**Scenario:** Order total is $0.30 (after discount). Stripe rejects (min $0.50). Customer sees generic error.

**Fix:** Validate `amount_cents >= 50` before calling Stripe.

### X4. Loyalty points never expire ❌ ACCOUNTING
**Scenario:** Customer accumulates 100,000 points over 3 years. Liability on books forever.

**Fix:** Add `expires_at` to loyalty points. Expire 12 months after earn. (Simplified: just document policy, add `expired_at` column.)

### X5. No review time window ❌ ABUSE
**Scenario:** Customer reviews 1 year after delivery. Restaurant can't verify.

**Fix:** Block reviews older than 30 days after delivery.

### X6. Frontend: AddItemModal allows adding when restaurant closed ❌ UX BUG
**Scenario:** Restaurant shows "Closed" badge, but "Add to Cart" button still works.

**Fix:** Disable add button when `!restaurant.is_open`.

### X7. Frontend: Checkout doesn't re-verify restaurant open ❌ UX BUG
**Scenario:** Customer starts checkout at 11:59 PM. Restaurant closes at midnight. Checkout still submits at 12:01 AM. Backend rejects with "restaurant is closed" — but customer already entered payment info.

**Fix:** Show warning in checkout if restaurant is closed or closing soon.

### X8. No notification on auto-reject ❌ POOR UX
**Scenario:** Order auto-rejected after 5 min. Customer only finds out by checking the tracking page.

**Fix:** Append `order.auto_rejected` event (already done) + add push notification (stub for now).

---

## Fix Priority

### P0 — Security bugs (fix immediately)
- C5: Cancel order ownership check
- R1/R2: Restaurant order action ownership checks
- D1: Driver pickup/deliver ownership check

### P1 — Abuse prevention + data integrity
- C2: Max quantity per item (99)
- C3: Max items per cart (50)
- C4: Max order value ($2000)
- C6: Duplicate order prevention (cart-derived idempotency)
- C9: Address lat/lng validation
- D2: Driver location throttle
- X2: WS subscription limit
- X3: Stripe min amount check

### P2 — Edge case completion
- C1: Cart blocks adding from closed restaurant
- C7: Price change detection (menu version)
- C8: Dispute time limit (7 days)
- D4: Vehicle type validation
- X1: Abandoned cart cleanup job
- X5: Review time window (30 days)
- X6/X7: Frontend closed-restaurant warnings

### P3 — UX polish
- C10: Password complexity
- D3: ETA recompute on driver assignment
- X8: Notification on auto-reject
- A2: Admin audit log

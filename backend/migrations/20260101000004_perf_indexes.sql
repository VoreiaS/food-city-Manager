-- =============================================================================
-- Performance indexes (Phase 8 hardening)
-- =============================================================================

-- Hot path: customer looks up their orders by customer_id, sorted by placed_at
-- Already indexed in 20260101000002, but let's add a covering index for the
-- common "list my recent orders" query.

-- Restaurant search by status + cuisine (already GIN on cuisine_types)
-- Add composite for "active + near coordinates"
CREATE INDEX IF NOT EXISTS restaurants_active_lat_lng_idx
    ON restaurants (lat, lng)
    WHERE deleted_at IS NULL AND status = 'active';

-- Order state machine queries: find pending_accept / ready for worker loops
CREATE INDEX IF NOT EXISTS orders_ready_no_driver_idx
    ON orders (ready_at ASC)
    WHERE status = 'ready' AND driver_id IS NULL;

CREATE INDEX IF NOT EXISTS orders_pending_accept_idx
    ON orders (placed_at ASC)
    WHERE status = 'pending_accept';

-- Driver availability queries
CREATE INDEX IF NOT EXISTS drivers_available_idx
    ON drivers (updated_at DESC)
    WHERE status = 'available';

-- Reviews by restaurant (already exists in 20260101000002, ensure)
CREATE INDEX IF NOT EXISTS reviews_restaurant_visible_idx
    ON reviews (restaurant_id, created_at DESC)
    WHERE is_hidden = false;

-- Disputes queue
CREATE INDEX IF NOT EXISTS disputes_open_idx
    ON disputes (created_at ASC)
    WHERE status = 'open';

-- Payment intent lookup by order (already exists in 20260101000002)

-- Order events replay: events_after(order_id, after_sequence)
-- Already covered by UNIQUE (order_id, sequence) + index.

-- Promo redemption lookup
CREATE INDEX IF NOT EXISTS promo_redemptions_user_idx
    ON promo_redemptions (user_id, redeemed_at DESC);

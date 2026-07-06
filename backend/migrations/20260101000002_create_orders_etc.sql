-- =============================================================================
-- CARTS + ORDERS
-- =============================================================================
CREATE TYPE cart_status AS ENUM ('active', 'locked', 'converted', 'abandoned');

CREATE TABLE carts (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    restaurant_id   UUID NOT NULL REFERENCES restaurants(id) ON DELETE CASCADE,
    status          cart_status NOT NULL DEFAULT 'active',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX carts_active_user_idx ON carts(user_id) WHERE status = 'active';

CREATE TABLE cart_items (
    id                      UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    cart_id                 UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    menu_item_id            UUID NOT NULL REFERENCES menu_items(id) ON DELETE CASCADE,
    menu_version_at_add     INTEGER NOT NULL,
    quantity                INTEGER NOT NULL CHECK (quantity > 0),
    customizations          JSONB NOT NULL DEFAULT '[]',
    notes                   TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX cart_items_cart_idx ON cart_items(cart_id);

-- Orders
CREATE TYPE order_status AS ENUM (
    'pending_accept', 'accepted', 'preparing', 'ready',
    'picked_up', 'delivering', 'delivered',
    'canceled', 'auto_rejected'
);

CREATE TYPE payment_status AS ENUM (
    'pending', 'succeeded', 'failed', 'canceled',
    'refunded', 'partially_refunded'
);

CREATE TABLE orders (
    id                      UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    customer_id             UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    restaurant_id           UUID NOT NULL REFERENCES restaurants(id) ON DELETE RESTRICT,
    driver_id               UUID REFERENCES users(id) ON DELETE SET NULL,
    status                  order_status NOT NULL DEFAULT 'pending_accept',
    payment_status          payment_status NOT NULL DEFAULT 'pending',
    -- Snapshot of cart at order time
    snapshot                JSONB NOT NULL,
    subtotal_cents          BIGINT NOT NULL,
    delivery_fee_cents      BIGINT NOT NULL DEFAULT 0,
    tax_cents               BIGINT NOT NULL DEFAULT 0,
    tip_cents               BIGINT NOT NULL DEFAULT 0,
    discount_cents          BIGINT NOT NULL DEFAULT 0,
    total_cents             BIGINT NOT NULL,
    currency                TEXT NOT NULL DEFAULT 'usd',
    -- Delivery address snapshot
    delivery_address        JSONB NOT NULL,
    notes                   TEXT,
    -- Timelines
    placed_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    accepted_at             TIMESTAMPTZ,
    preparing_at            TIMESTAMPTZ,
    ready_at                TIMESTAMPTZ,
    picked_up_at            TIMESTAMPTZ,
    delivered_at            TIMESTAMPTZ,
    canceled_at             TIMESTAMPTZ,
    cancellation_reason     TEXT,
    estimated_delivery_at   TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX orders_customer_idx ON orders(customer_id) ORDER BY placed_at DESC;
CREATE INDEX orders_restaurant_idx ON orders(restaurant_id) ORDER BY placed_at DESC;
CREATE INDEX orders_driver_idx ON orders(driver_id) WHERE driver_id IS NOT NULL;
CREATE INDEX orders_status_idx ON orders(status);

CREATE TABLE order_items (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    order_id        UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    menu_item_id    UUID,  -- nullable if menu item later deleted
    name            TEXT NOT NULL,
    description     TEXT,
    price_cents     BIGINT NOT NULL,
    quantity        INTEGER NOT NULL CHECK (quantity > 0),
    customizations  JSONB NOT NULL DEFAULT '[]',
    notes           TEXT,
    status          TEXT NOT NULL DEFAULT 'pending'
);

CREATE INDEX order_items_order_idx ON order_items(order_id);

-- Order events (for WS replay)
CREATE TABLE order_events (
    id          BIGSERIAL PRIMARY KEY,
    order_id    UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    sequence    BIGINT NOT NULL,
    event_type  TEXT NOT NULL,
    payload     JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (order_id, sequence)
);

CREATE INDEX order_events_order_seq_idx ON order_events(order_id, sequence);

-- =============================================================================
-- DRIVERS
-- =============================================================================
CREATE TYPE driver_status AS ENUM (
    'offline', 'available', 'assigned', 'en_route',
    'at_restaurant', 'picked_up', 'delivering', 'delivered'
);

CREATE TABLE drivers (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id             UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    vehicle_type        TEXT NOT NULL,
    license_plate       TEXT,
    current_lat         DOUBLE PRECISION,
    current_lng         DOUBLE PRECISION,
    status              driver_status NOT NULL DEFAULT 'offline',
    current_order_id    UUID REFERENCES orders(id) ON DELETE SET NULL,
    rating_avg          DOUBLE PRECISION,
    rating_count        BIGINT NOT NULL DEFAULT 0,
    acceptance_rate     DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    total_deliveries    BIGINT NOT NULL DEFAULT 0,
    stripe_account_id   TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX drivers_status_idx ON drivers(status) WHERE status != 'offline';
CREATE INDEX drivers_user_idx ON drivers(user_id);

CREATE TABLE delivery_proofs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    order_id    UUID NOT NULL UNIQUE REFERENCES orders(id) ON DELETE CASCADE,
    driver_id   UUID NOT NULL REFERENCES drivers(id) ON DELETE RESTRICT,
    photo_url   TEXT,
    gps_lat     DOUBLE PRECISION NOT NULL,
    gps_lng     DOUBLE PRECISION NOT NULL,
    otp_hash    TEXT,
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- PAYMENTS
-- =============================================================================
CREATE TABLE payment_intents (
    id                      UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    order_id                UUID NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    provider                TEXT NOT NULL DEFAULT 'stripe',
    provider_intent_id      TEXT,
    idempotency_key         TEXT NOT NULL UNIQUE,
    amount_cents            BIGINT NOT NULL,
    currency                TEXT NOT NULL DEFAULT 'usd',
    status                  payment_status NOT NULL DEFAULT 'pending',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX payment_intents_order_idx ON payment_intents(order_id);

CREATE TABLE payment_webhooks (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    provider_event_id   TEXT NOT NULL UNIQUE,
    event_type          TEXT NOT NULL,
    payload             JSONB NOT NULL,
    processed_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE payout_ledger (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    order_id            UUID NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
    payee_type          TEXT NOT NULL CHECK (payee_type IN ('restaurant', 'driver', 'platform')),
    payee_id            UUID NOT NULL,
    amount_cents        BIGINT NOT NULL,
    currency            TEXT NOT NULL DEFAULT 'usd',
    stripe_transfer_id  TEXT,
    status              TEXT NOT NULL DEFAULT 'pending',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paid_at             TIMESTAMPTZ
);

CREATE INDEX payout_ledger_payee_idx ON payout_ledger(payee_type, payee_id);
CREATE INDEX payout_ledger_order_idx ON payout_ledger(order_id);

-- =============================================================================
-- REVIEWS
-- =============================================================================
CREATE TABLE reviews (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    order_id            UUID NOT NULL UNIQUE REFERENCES orders(id) ON DELETE CASCADE,
    customer_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    restaurant_id       UUID NOT NULL REFERENCES restaurants(id) ON DELETE CASCADE,
    rating_food         SMALLINT NOT NULL CHECK (rating_food BETWEEN 1 AND 5),
    rating_delivery     SMALLINT NOT NULL CHECK (rating_delivery BETWEEN 1 AND 5),
    rating_packaging    SMALLINT NOT NULL CHECK (rating_packaging BETWEEN 1 AND 5),
    rating_overall      SMALLINT NOT NULL CHECK (rating_overall BETWEEN 1 AND 5),
    body                TEXT,
    photo_urls          TEXT[] NOT NULL DEFAULT '{}',
    reply_body          TEXT,
    reply_at            TIMESTAMPTZ,
    is_hidden           BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX reviews_restaurant_idx ON reviews(restaurant_id, created_at DESC) WHERE is_hidden = false;
CREATE INDEX reviews_customer_idx ON reviews(customer_id);

-- =============================================================================
-- LOYALTY
-- =============================================================================
CREATE TYPE loyalty_tier AS ENUM ('silver', 'gold', 'platinum');

CREATE TABLE loyalty_accounts (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id             UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    points_balance      BIGINT NOT NULL DEFAULT 0,
    tier                loyalty_tier NOT NULL DEFAULT 'silver',
    lifetime_points     BIGINT NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE loyalty_transactions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    account_id  UUID NOT NULL REFERENCES loyalty_accounts(id) ON DELETE CASCADE,
    points_delta BIGINT NOT NULL,  -- positive = earn, negative = redeem/refund
    reason      TEXT NOT NULL,
    order_id    UUID REFERENCES orders(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX loyalty_transactions_account_idx ON loyalty_transactions(account_id, created_at DESC);

-- =============================================================================
-- PROMO CODES
-- =============================================================================
CREATE TABLE promo_codes (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    code            TEXT NOT NULL UNIQUE,
    description     TEXT,
    discount_type   TEXT NOT NULL CHECK (discount_type IN ('percent', 'flat', 'free_delivery')),
    discount_value  NUMERIC(10,2) NOT NULL DEFAULT 0,  -- percent (0-100) or flat cents
    min_order_cents BIGINT NOT NULL DEFAULT 0,
    max_uses        INTEGER,
    used_count      INTEGER NOT NULL DEFAULT 0,
    daily_cap       INTEGER,
    per_user_cap    INTEGER NOT NULL DEFAULT 1,
    valid_from      TIMESTAMPTZ NOT NULL,
    valid_until     TIMESTAMPTZ,
    active          BOOLEAN NOT NULL DEFAULT true,
    applicable_restaurants UUID[] NOT NULL DEFAULT '{}',  -- empty = all
    customer_segment TEXT NOT NULL DEFAULT 'all',  -- all | new | vip
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE promo_redemptions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    promo_code_id   UUID NOT NULL REFERENCES promo_codes(id) ON DELETE RESTRICT,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    order_id        UUID REFERENCES orders(id) ON DELETE SET NULL,
    redeemed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (promo_code_id, user_id)
);

-- =============================================================================
-- DISPUTES
-- =============================================================================
CREATE TYPE dispute_status AS ENUM ('open', 'resolved', 'rejected', 'escalated');

CREATE TABLE disputes (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    order_id            UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    customer_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    issue_type          TEXT NOT NULL,
    description         TEXT NOT NULL,
    evidence_urls       TEXT[] NOT NULL DEFAULT '{}',
    status              dispute_status NOT NULL DEFAULT 'open',
    resolution          TEXT,
    refund_amount_cents BIGINT,
    resolved_by         UUID REFERENCES users(id),
    resolved_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX disputes_status_idx ON disputes(status, created_at DESC);
CREATE INDEX disputes_customer_idx ON disputes(customer_id);
CREATE INDEX disputes_order_idx ON disputes(order_id);

-- =============================================================================
-- updated_at triggers (auto-update on row change)
-- =============================================================================
CREATE OR REPLACE FUNCTION touch_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to all tables with updated_at
DO $$
DECLARE
    t TEXT;
BEGIN
    FOR t IN
        SELECT table_name FROM information_schema.columns
        WHERE column_name = 'updated_at'
        AND table_schema = 'public'
    LOOP
        EXECUTE format('
            CREATE TRIGGER set_updated_at
            BEFORE UPDATE ON %I
            FOR EACH ROW
            WHEN (OLD.updated_at IS DISTINCT FROM NEW.updated_at OR OLD.updated_at = NEW.updated_at)
            EXECUTE FUNCTION touch_updated_at();
        ', t);
    END LOOP;
END;
$$;

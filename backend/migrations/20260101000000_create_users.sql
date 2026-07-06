-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Note: PostGIS extension is needed for `GEOGRAPHY(POINT)` columns.
-- Enable it once PostGIS is installed in the Docker image.
-- CREATE EXTENSION IF NOT EXISTS postgis;

-- =============================================================================
-- USERS
-- =============================================================================
CREATE TYPE user_role AS ENUM ('customer', 'restaurant', 'driver', 'admin');

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    email           TEXT NOT NULL UNIQUE,
    phone           TEXT NOT NULL UNIQUE,
    full_name       TEXT NOT NULL,
    role            user_role NOT NULL,
    password_hash   TEXT NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX users_role_idx ON users(role) WHERE deleted_at IS NULL;
CREATE INDEX users_email_idx ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX users_created_at_idx ON users(created_at DESC);

COMMENT ON TABLE users IS 'All platform users (customer/restaurant/driver/admin).';
COMMENT ON COLUMN users.password_hash IS 'argon2 hash, never returned to client.';

-- =============================================================================
-- ADDRESSES
-- =============================================================================
CREATE TABLE addresses (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label               TEXT NOT NULL,
    line1               TEXT NOT NULL,
    line2               TEXT,
    city                TEXT NOT NULL,
    postal_code         TEXT,
    lat                 DOUBLE PRECISION NOT NULL,
    lng                 DOUBLE PRECISION NOT NULL,
    formatted_address   TEXT NOT NULL,
    is_default          BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX addresses_user_idx ON addresses(user_id) WHERE deleted_at IS NULL;
CREATE INDEX addresses_lat_lng_idx ON addresses(lat, lng);

-- =============================================================================
-- RESTAURANT GROUPS (for chains)
-- =============================================================================
CREATE TABLE restaurant_groups (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    name        TEXT NOT NULL,
    logo_url    TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- RESTAURANTS
-- =============================================================================
CREATE TYPE restaurant_status AS ENUM (
    'pending_verification', 'active', 'paused', 'closing', 'closed'
);

CREATE TABLE restaurants (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    owner_user_id       UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    group_id            UUID REFERENCES restaurant_groups(id) ON DELETE SET NULL,
    name                TEXT NOT NULL,
    slug                TEXT NOT NULL UNIQUE,
    description         TEXT,
    cuisine_types       TEXT[] NOT NULL DEFAULT '{}',
    price_range         SMALLINT NOT NULL DEFAULT 2 CHECK (price_range BETWEEN 1 AND 4),
    logo_url            TEXT,
    cover_url           TEXT,
    -- GEOGRAPHY(POINT) requires PostGIS. Use plain DOUBLE PRECISION for now.
    lat                 DOUBLE PRECISION NOT NULL,
    lng                 DOUBLE PRECISION NOT NULL,
    delivery_radius_m   INTEGER NOT NULL DEFAULT 5000,
    delivery_fee_cents  BIGINT NOT NULL DEFAULT 0,
    min_order_cents     BIGINT NOT NULL DEFAULT 0,
    status              restaurant_status NOT NULL DEFAULT 'pending_verification',
    hours_json          JSONB NOT NULL DEFAULT '{}',
    rating_avg          DOUBLE PRECISION,
    rating_count        BIGINT NOT NULL DEFAULT 0,
    -- Stripe Connect account
    stripe_account_id   TEXT,
    commission_percent  NUMERIC(5,2) NOT NULL DEFAULT 15.00,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX restaurants_owner_idx ON restaurants(owner_user_id) WHERE deleted_at IS NULL;
CREATE INDEX restaurants_status_idx ON restaurants(status) WHERE deleted_at IS NULL;
CREATE INDEX restaurants_slug_idx ON restaurants(slug) WHERE deleted_at IS NULL;
CREATE INDEX restaurants_lat_lng_idx ON restaurants(lat, lng);
CREATE INDEX restaurants_cuisine_idx ON restaurants USING GIN (cuisine_types);

-- =============================================================================
-- RESTAURANT HOURS EXCEPTIONS
-- =============================================================================
CREATE TABLE restaurant_hours_exceptions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    restaurant_id   UUID NOT NULL REFERENCES restaurants(id) ON DELETE CASCADE,
    date            DATE NOT NULL,
    is_closed       BOOLEAN NOT NULL DEFAULT false,
    open_time       TIME,
    close_time      TIME,
    notes           TEXT,
    UNIQUE (restaurant_id, date)
);

-- =============================================================================
-- RESTAURANT VERIFICATIONS
-- =============================================================================
CREATE TYPE verification_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE restaurant_verifications (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    restaurant_id   UUID NOT NULL REFERENCES restaurants(id) ON DELETE CASCADE,
    status          verification_status NOT NULL DEFAULT 'pending',
    documents       JSONB NOT NULL DEFAULT '{}',
    reviewed_by     UUID REFERENCES users(id),
    reviewed_at     TIMESTAMPTZ,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX verifications_status_idx ON restaurant_verifications(status);
CREATE INDEX verifications_restaurant_idx ON restaurant_verifications(restaurant_id);

-- =============================================================================
-- MENUS (versioned)
-- =============================================================================
CREATE TABLE menu_versions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    restaurant_id   UUID NOT NULL REFERENCES restaurants(id) ON DELETE CASCADE,
    version         INTEGER NOT NULL,
    published_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    UNIQUE (restaurant_id, version)
);

CREATE INDEX menu_versions_restaurant_idx ON menu_versions(restaurant_id) WHERE is_active = true;

CREATE TABLE menu_categories (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    menu_version_id     UUID NOT NULL REFERENCES menu_versions(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    sort_order          INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX menu_categories_version_idx ON menu_categories(menu_version_id);

CREATE TYPE menu_item_status AS ENUM ('available', 'out_of_stock', 'hidden');

CREATE TABLE menu_items (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    category_id         UUID NOT NULL REFERENCES menu_categories(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    description         TEXT,
    price_cents         BIGINT NOT NULL CHECK (price_cents >= 0),
    image_url           TEXT,
    is_veg              BOOLEAN NOT NULL DEFAULT false,
    is_vegan            BOOLEAN NOT NULL DEFAULT false,
    is_halal            BOOLEAN NOT NULL DEFAULT false,
    spice_level         SMALLINT NOT NULL DEFAULT 0 CHECK (spice_level BETWEEN 0 AND 3),
    allergens           TEXT[] NOT NULL DEFAULT '{}',
    track_stock         BOOLEAN NOT NULL DEFAULT false,
    stock_count         INTEGER NOT NULL DEFAULT 0,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    status              menu_item_status NOT NULL DEFAULT 'available',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX menu_items_category_idx ON menu_items(category_id);
CREATE INDEX menu_items_status_idx ON menu_items(status);
CREATE INDEX menu_items_allergens_idx ON menu_items USING GIN (allergens);

CREATE TABLE menu_item_customizations (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    item_id     UUID NOT NULL REFERENCES menu_items(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT false,
    max_select  INTEGER,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE menu_item_customization_options (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    customization_id    UUID NOT NULL REFERENCES menu_item_customizations(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    price_cents         BIGINT NOT NULL DEFAULT 0,
    is_default          BOOLEAN NOT NULL DEFAULT false,
    sort_order          INTEGER NOT NULL DEFAULT 0
);

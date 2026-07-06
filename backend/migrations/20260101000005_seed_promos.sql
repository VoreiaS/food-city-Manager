-- =============================================================================
-- Seed: sample promo codes for development
-- =============================================================================

INSERT INTO promo_codes (code, description, discount_type, discount_value, min_order_cents, max_uses, per_user_cap, valid_from, valid_until, active, customer_segment)
VALUES (
    'WELCOME20',
    '20% off your first order (max $10)',
    'percent',
    20.00,
    1500,
    1000,
    1,
    NOW(),
    NOW() + INTERVAL '365 days',
    true,
    'all'
) ON CONFLICT (code) DO NOTHING;

INSERT INTO promo_codes (code, description, discount_type, discount_value, min_order_cents, max_uses, per_user_cap, valid_from, valid_until, active, customer_segment)
VALUES (
    'FREEDEL',
    'Free delivery on orders over $25',
    'free_delivery',
    0,
    2500,
    5000,
    5,
    NOW(),
    NOW() + INTERVAL '90 days',
    true,
    'all'
) ON CONFLICT (code) DO NOTHING;

INSERT INTO promo_codes (code, description, discount_type, discount_value, min_order_cents, max_uses, per_user_cap, valid_from, valid_until, active, customer_segment)
VALUES (
    'SAVE5',
    '$5 off any order over $20',
    'flat',
    500,
    2000,
    NULL,
    3,
    NOW(),
    NULL,
    true,
    'all'
) ON CONFLICT (code) DO NOTHING;

-- =============================================================================
-- Seed data: sample restaurants + menus for development.
-- Safe to re-run (uses ON CONFLICT).
-- =============================================================================

-- Owner user for seed restaurants (password: "password123")
-- Hash generated with argon2 — using a known-fixed hash for dev.
INSERT INTO users (id, email, phone, full_name, role, password_hash, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'owner@spicevilla.example',
    '+15550000001',
    'Spice Villa Owner',
    'restaurant',
    '$argon2id$v=19$m=19456,t=2,p=1$nIdjYhLF2fTjEBE+FqP1xw$Y8lLqHlmh8QXa1h5Nk4qL+UdTPv0wPwU8tQwUH3rELk',
    true
) ON CONFLICT (email) DO NOTHING;

INSERT INTO users (id, email, phone, full_name, role, password_hash, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000002',
    'owner@pizzahub.example',
    '+15550000002',
    'Pizza Hub Owner',
    'restaurant',
    '$argon2id$v=19$m=19456,t=2,p=1$nIdjYhLF2fTjEBE+FqP1xw$Y8lLqHlmh8QXa1h5Nk4qL+UdTPv0wPwU8tQwUH3rELk',
    true
) ON CONFLICT (email) DO NOTHING;

INSERT INTO users (id, email, phone, full_name, role, password_hash, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000003',
    'owner@sushiworld.example',
    '+15550000003',
    'Sushi World Owner',
    'restaurant',
    '$argon2id$v=19$m=19456,t=2,p=1$nIdjYhLF2fTjEBE+FqP1xw$Y8lLqHlmh8QXa1h5Nk4qL+UdTPv0wPwU8tQwUH3rELk',
    true
) ON CONFLICT (email) DO NOTHING;

-- =============================================================================
-- Restaurants (3 samples in the same city block)
-- =============================================================================

INSERT INTO restaurants (id, owner_user_id, name, slug, description, cuisine_types, price_range, lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents, status, hours_json, rating_avg, rating_count)
VALUES (
    '00000000-0000-1000-0000-000000000001',
    '00000000-0000-0000-0000-000000000001',
    'Spice Villa',
    'spice-villa',
    'Authentic Indian cuisine with a modern twist. Famous for our biryanis and tandoori.',
    ARRAY['indian', 'vegetarian'],
    2,
    6.9271, 79.8612,
    5000, 250, 1000,
    'active',
    '{"monday":[{"open":"11:00","close":"22:00"}],"tuesday":[{"open":"11:00","close":"22:00"}],"wednesday":[{"open":"11:00","close":"22:00"}],"thursday":[{"open":"11:00","close":"22:00"}],"friday":[{"open":"11:00","close":"23:00"}],"saturday":[{"open":"11:00","close":"23:00"}],"sunday":[{"open":"12:00","close":"22:00"}]}'::jsonb,
    4.6, 1243
) ON CONFLICT (slug) DO NOTHING;

INSERT INTO restaurants (id, owner_user_id, name, slug, description, cuisine_types, price_range, lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents, status, hours_json, rating_avg, rating_count)
VALUES (
    '00000000-0000-1000-0000-000000000002',
    '00000000-0000-0000-0000-000000000002',
    'Pizza Hub',
    'pizza-hub',
    'Wood-fired Neapolitan pizzas, fresh pasta, and Italian antipasti.',
    ARRAY['italian', 'pizza'],
    2,
    6.9275, 79.8618,
    6000, 300, 1500,
    'active',
    '{"monday":[{"open":"10:00","close":"23:00"}],"tuesday":[{"open":"10:00","close":"23:00"}],"wednesday":[{"open":"10:00","close":"23:00"}],"thursday":[{"open":"10:00","close":"23:00"}],"friday":[{"open":"10:00","close":"00:00"}],"saturday":[{"open":"10:00","close":"00:00"}],"sunday":[{"open":"12:00","close":"23:00"}]}'::jsonb,
    4.4, 892
) ON CONFLICT (slug) DO NOTHING;

INSERT INTO restaurants (id, owner_user_id, name, slug, description, cuisine_types, price_range, lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents, status, hours_json, rating_avg, rating_count)
VALUES (
    '00000000-0000-1000-0000-000000000003',
    '00000000-0000-0000-0000-000000000003',
    'Sushi World',
    'sushi-world',
    'Omakase-style sushi, ramen, and Japanese small plates.',
    ARRAY['japanese', 'sushi'],
    3,
    6.9280, 79.8605,
    4500, 350, 2000,
    'active',
    '{"monday":[{"open":"17:00","close":"22:30"}],"tuesday":[{"open":"17:00","close":"22:30"}],"wednesday":[{"open":"17:00","close":"22:30"}],"thursday":[{"open":"17:00","close":"22:30"}],"friday":[{"open":"17:00","close":"23:30"}],"saturday":[{"open":"12:00","close":"23:30"}],"sunday":[{"open":"12:00","close":"22:00"}]}'::jsonb,
    4.8, 567
) ON CONFLICT (slug) DO NOTHING;

-- =============================================================================
-- Menu for Spice Villa
-- =============================================================================
INSERT INTO menu_versions (id, restaurant_id, version, is_active)
VALUES (
    '00000000-0000-2000-0000-000000000001',
    '00000000-0000-1000-0000-000000000001',
    1, true
) ON CONFLICT DO NOTHING;

INSERT INTO menu_categories (id, menu_version_id, name, sort_order) VALUES
('00000000-0000-3000-0000-000000000001', '00000000-0000-2000-0000-000000000001', 'Starters', 1),
('00000000-0000-3000-0000-000000000002', '00000000-0000-2000-0000-000000000001', 'Mains', 2),
('00000000-0000-3000-0000-000000000003', '00000000-0000-2000-0000-000000000001', 'Breads & Rice', 3),
('00000000-0000-3000-0000-000000000004', '00000000-0000-2000-0000-000000000001', 'Desserts', 4)
ON CONFLICT DO NOTHING;

INSERT INTO menu_items (id, category_id, name, description, price_cents, is_veg, spice_level, allergens, sort_order, status) VALUES
('00000000-0000-4000-0000-000000000001', '00000000-0000-3000-0000-000000000001', 'Paneer Tikka', 'Cottage cheese marinated in spiced yogurt, char-grilled.', 850, true, 2, ARRAY['dairy'], 1, 'available'),
('00000000-0000-4000-0000-000000000002', '00000000-0000-3000-0000-000000000001', 'Samosa (2 pc)', 'Crispy pastry filled with spiced potatoes and peas.', 450, true, 1, ARRAY['gluten'], 2, 'available'),
('00000000-0000-4000-0000-000000000003', '00000000-0000-3000-0000-000000000001', 'Chicken 65', 'Deep-fried chicken with curry leaves and red chilies.', 950, false, 3, ARRAY[], 3, 'available'),
('00000000-0000-4000-0000-000000000004', '00000000-0000-3000-0000-000000000002', 'Butter Chicken', 'Tandoori chicken simmered in creamy tomato gravy.', 1450, false, 1, ARRAY['dairy'], 1, 'available'),
('00000000-0000-4000-0000-000000000005', '00000000-0000-3000-0000-000000000002', 'Paneer Makhani', 'Cottage cheese in a rich tomato-cashew gravy.', 1350, true, 1, ARRAY['dairy', 'nuts'], 2, 'available'),
('00000000-0000-4000-0000-000000000006', '00000000-0000-3000-0000-000000000002', 'Dal Makhani', 'Black lentils slow-cooked overnight with butter.', 1100, true, 0, ARRAY['dairy'], 3, 'available'),
('00000000-0000-4000-0000-000000000007', '00000000-0000-3000-0000-000000000003', 'Hyderabadi Biryani', 'Long-grain basmati layered with spiced meat, sealed and dum-cooked.', 1650, false, 2, ARRAY[], 1, 'available'),
('00000000-0000-4000-0000-000000000008', '00000000-0000-3000-0000-000000000003', 'Veg Biryani', 'Fragrant basmati with mixed vegetables and saffron.', 1350, true, 2, ARRAY[], 2, 'available'),
('00000000-0000-4000-0000-000000000009', '00000000-0000-3000-0000-000000000003', 'Garlic Naan', 'Tandoor-baked flatbread brushed with garlic butter.', 250, true, 0, ARRAY['gluten', 'dairy'], 3, 'available'),
('00000000-0000-4000-0000-000000000010', '00000000-0000-3000-0000-000000000004', 'Gulab Jamun (2 pc)', 'Warm milk dumplings in rose-scented syrup.', 350, true, 0, ARRAY['dairy'], 1, 'available')
ON CONFLICT DO NOTHING;

-- Customizations for biryani: spice level
INSERT INTO menu_item_customizations (id, item_id, name, is_required, sort_order) VALUES
('00000000-0000-5000-0000-000000000001', '00000000-0000-4000-0000-000000000007', 'Spice Level', true, 1),
('00000000-0000-5000-0000-000000000002', '00000000-0000-4000-0000-000000000007', 'Raita', false, 2)
ON CONFLICT DO NOTHING;

INSERT INTO menu_item_customization_options (id, customization_id, name, price_cents, is_default, sort_order) VALUES
('00000000-0000-6000-0000-000000000001', '00000000-0000-5000-0000-000000000001', 'Mild', 0, true, 1),
('00000000-0000-6000-0000-000000000002', '00000000-0000-5000-0000-000000000001', 'Medium', 0, false, 2),
('00000000-0000-6000-0000-000000000003', '00000000-0000-5000-0000-000000000001', 'Hot', 50, false, 3),
('00000000-0000-6000-0000-000000000004', '00000000-0000-5000-0000-000000000002', 'No raita', 0, true, 1),
('00000000-0000-6000-0000-000000000005', '00000000-0000-5000-0000-000000000002', 'Add raita', 75, false, 2)
ON CONFLICT DO NOTHING;

-- =============================================================================
-- Menu for Pizza Hub
-- =============================================================================
INSERT INTO menu_versions (id, restaurant_id, version, is_active)
VALUES (
    '00000000-0000-2000-0000-000000000002',
    '00000000-0000-1000-0000-000000000002',
    1, true
) ON CONFLICT DO NOTHING;

INSERT INTO menu_categories (id, menu_version_id, name, sort_order) VALUES
('00000000-0000-3000-0000-000000000005', '00000000-0000-2000-0000-000000000002', 'Pizzas', 1),
('00000000-0000-3000-0000-000000000006', '00000000-0000-2000-0000-000000000002', 'Pasta', 2),
('00000000-0000-3000-0000-000000000007', '00000000-0000-2000-0000-000000000002', 'Sides', 3)
ON CONFLICT DO NOTHING;

INSERT INTO menu_items (id, category_id, name, description, price_cents, is_veg, spice_level, allergens, sort_order, status) VALUES
('00000000-0000-4000-0000-000000000011', '00000000-0000-3000-0000-000000000005', 'Margherita', 'San Marzano tomato, fresh mozzarella, basil.', 1200, true, 0, ARRAY['gluten', 'dairy'], 1, 'available'),
('00000000-0000-4000-0000-000000000012', '00000000-0000-3000-0000-000000000005', 'Pepperoni', 'Tomato, mozzarella, double pepperoni.', 1550, false, 0, ARRAY['gluten', 'dairy'], 2, 'available'),
('00000000-0000-4000-0000-000000000013', '00000000-0000-3000-0000-000000000005', 'Quattro Formaggi', 'Mozzarella, gorgonzola, fontina, parmesan.', 1750, true, 0, ARRAY['gluten', 'dairy'], 3, 'available'),
('00000000-0000-4000-0000-000000000014', '00000000-0000-3000-0000-000000000006', 'Spaghetti Carbonara', 'Pancetta, egg, pecorino, black pepper.', 1450, false, 0, ARRAY['gluten', 'egg', 'dairy'], 1, 'available'),
('00000000-0000-4000-0000-000000000015', '00000000-0000-3000-0000-000000000006', 'Penne Arrabbiata', 'Tomato, garlic, chili, parsley.', 1250, true, 2, ARRAY['gluten'], 2, 'available'),
('00000000-0000-4000-0000-000000000016', '00000000-0000-3000-0000-000000000007', 'Garlic Bread', 'Toasted ciabatta with garlic butter.', 550, true, 0, ARRAY['gluten', 'dairy'], 1, 'available'),
('00000000-0000-4000-0000-000000000017', '00000000-0000-3000-0000-000000000007', 'Caesar Salad', 'Romaine, croutons, parmesan, caesar dressing.', 850, true, 0, ARRAY['gluten', 'dairy', 'egg', 'fish'], 2, 'available')
ON CONFLICT DO NOTHING;

-- Pizza size customization
INSERT INTO menu_item_customizations (id, item_id, name, is_required, sort_order) VALUES
('00000000-0000-5000-0000-000000000003', '00000000-0000-4000-0000-000000000011', 'Size', true, 1),
('00000000-0000-5000-0000-000000000004', '00000000-0000-4000-0000-000000000012', 'Size', true, 1),
('00000000-0000-5000-0000-000000000005', '00000000-0000-4000-0000-000000000013', 'Size', true, 1)
ON CONFLICT DO NOTHING;

INSERT INTO menu_item_customization_options (id, customization_id, name, price_cents, is_default, sort_order) VALUES
('00000000-0000-6000-0000-000000000006', '00000000-0000-5000-0000-000000000003', '10" Small', 0, true, 1),
('00000000-0000-6000-0000-000000000007', '00000000-0000-5000-0000-000000000003', '14" Medium', 400, false, 2),
('00000000-0000-6000-0000-000000000008', '00000000-0000-5000-0000-000000000003', '18" Large', 800, false, 3),
('00000000-0000-6000-0000-000000000009', '00000000-0000-5000-0000-000000000004', '10" Small', 0, true, 1),
('00000000-0000-6000-0000-000000000010', '00000000-0000-5000-0000-000000000004', '14" Medium', 400, false, 2),
('00000000-0000-6000-0000-000000000011', '00000000-0000-5000-0000-000000000004', '18" Large', 800, false, 3),
('00000000-0000-6000-0000-000000000012', '00000000-0000-5000-0000-000000000005', '10" Small', 0, true, 1),
('00000000-0000-6000-0000-000000000013', '00000000-0000-5000-0000-000000000005', '14" Medium', 400, false, 2),
('00000000-0000-6000-0000-000000000014', '00000000-0000-5000-0000-000000000005', '18" Large', 800, false, 3)
ON CONFLICT DO NOTHING;

-- =============================================================================
-- Menu for Sushi World
-- =============================================================================
INSERT INTO menu_versions (id, restaurant_id, version, is_active)
VALUES (
    '00000000-0000-2000-0000-000000000003',
    '00000000-0000-1000-0000-000000000003',
    1, true
) ON CONFLICT DO NOTHING;

INSERT INTO menu_categories (id, menu_version_id, name, sort_order) VALUES
('00000000-0000-3000-0000-000000000008', '00000000-0000-2000-0000-000000000003', 'Nigiri & Sashimi', 1),
('00000000-0000-3000-0000-000000000009', '00000000-0000-2000-0000-000000000003', 'Rolls', 2),
('00000000-0000-3000-0000-000000000010', '00000000-0000-2000-0000-000000000003', 'Ramen', 3)
ON CONFLICT DO NOTHING;

INSERT INTO menu_items (id, category_id, name, description, price_cents, is_veg, spice_level, allergens, sort_order, status) VALUES
('00000000-0000-4000-0000-000000000018', '00000000-0000-3000-0000-000000000008', 'Salmon Nigiri (2 pc)', 'Fresh Atlantic salmon over seasoned rice.', 600, false, 0, ARRAY['fish'], 1, 'available'),
('00000000-0000-4000-0000-000000000019', '00000000-0000-3000-0000-000000000008', 'Tuna Sashimi (5 pc)', 'Sliced bluefin tuna, no rice.', 950, false, 0, ARRAY['fish'], 2, 'available'),
('00000000-0000-4000-0000-000000000020', '00000000-0000-3000-0000-000000000009', 'California Roll (8 pc)', 'Crab, avocado, cucumber, sesame.', 850, false, 0, ARRAY['fish', 'sesame'], 1, 'available'),
('00000000-0000-4000-0000-000000000021', '00000000-0000-3000-0000-000000000009', 'Spicy Tuna Roll (8 pc)', 'Tuna, spicy mayo, scallions.', 950, false, 2, ARRAY['fish', 'egg'], 2, 'available'),
('00000000-0000-4000-0000-000000000022', '00000000-0000-3000-0000-000000000009', 'Vegetable Roll (8 pc)', 'Cucumber, avocado, carrot, asparagus.', 750, true, 0, ARRAY['sesame'], 3, 'available'),
('00000000-0000-4000-0000-000000000023', '00000000-0000-3000-0000-000000000010', 'Tonkotsu Ramen', 'Pork bone broth, chashu, egg, scallions, noodles.', 1650, false, 1, ARRAY['gluten', 'egg', 'soy'], 1, 'available'),
('00000000-0000-4000-0000-000000000024', '00000000-0000-3000-0000-000000000010', 'Miso Vegetable Ramen', 'Miso broth, tofu, vegetables, noodles.', 1450, true, 0, ARRAY['gluten', 'soy'], 2, 'available')
ON CONFLICT DO NOTHING;

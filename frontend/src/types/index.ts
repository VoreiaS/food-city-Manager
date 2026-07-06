// Shared TypeScript types — mirror backend domain/.

export type UserRole = "customer" | "restaurant" | "driver" | "admin";

export interface UserPublic {
  id: string;
  email: string;
  phone: string;
  full_name: string;
  role: UserRole;
  created_at: string;
}

export interface AuthResponse {
  user: UserPublic;
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
    details?: unknown;
    request_id?: string;
  };
}

export type RestaurantStatus =
  | "pending_verification"
  | "active"
  | "paused"
  | "closing"
  | "closed";

export interface Restaurant {
  id: string;
  owner_user_id: string;
  group_id: string | null;
  name: string;
  slug: string;
  description?: string;
  cuisine_types: string[];
  price_range: number;
  logo_url: string | null;
  cover_url: string | null;
  lat: number;
  lng: number;
  delivery_radius_m: number;
  delivery_fee_cents: number;
  min_order_cents: number;
  status: RestaurantStatus;
  hours_json: Record<string, Array<{ open: string; close: string }>>;
  rating_avg: number | null;
  rating_count: number;
  created_at: string;
  updated_at: string;
}

export interface MenuItemCustomizationOption {
  id: string;
  name: string;
  price_cents: number;
  is_default: boolean;
  sort_order: number;
}

export interface MenuItemCustomization {
  id: string;
  name: string;
  is_required: boolean;
  max_select: number | null;
  sort_order: number;
  options: MenuItemCustomizationOption[];
}

export interface MenuItem {
  id: string;
  name: string;
  description?: string;
  price_cents: number;
  image_url?: string;
  is_veg: boolean;
  is_vegan: boolean;
  is_halal: boolean;
  spice_level: number;
  allergens: string[];
  in_stock: boolean;
  sort_order: number;
  customizations: MenuItemCustomization[];
}

export interface MenuCategory {
  id: string;
  name: string;
  sort_order: number;
  items: MenuItem[];
}

export interface MenuResponse {
  restaurant_id: string;
  menu_version: number;
  categories: MenuCategory[];
}

// Re-export OrderDto as a type so other modules can import from @/types.
import type { OrderDto } from "@/api/orders.api";
export type { OrderDto };

export type OrderStatus =
  | "pending_accept"
  | "accepted"
  | "preparing"
  | "ready"
  | "picked_up"
  | "delivering"
  | "delivered"
  | "canceled"
  | "auto_rejected";

export interface Order {
  id: string;
  customer_id: string;
  restaurant_id: string;
  driver_id: string | null;
  status: OrderStatus;
  total_cents: number;
  currency: string;
  placed_at: string;
  estimated_delivery_at: string | null;
}

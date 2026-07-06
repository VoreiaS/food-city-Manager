import { apiClient } from "./client";

export interface CartItemCustomization {
  customization_id: string;
  customization_name: string;
  option_id: string;
  option_name: string;
  price_cents: number;
}

export interface CartItem {
  id: string;
  cart_id: string;
  menu_item_id: string;
  menu_item_name: string;
  menu_item_image_url: string | null;
  base_price_cents: number;
  quantity: number;
  customizations: CartItemCustomization[];
  notes: string | null;
  line_total_cents: number;
}

export interface CartResponse {
  id: string;
  user_id: string;
  restaurant_id: string;
  restaurant_name: string;
  status: "active" | "locked" | "converted" | "abandoned";
  items: CartItem[];
  subtotal_cents: number;
  delivery_fee_cents: number;
  total_cents: number;
  min_order_cents: number;
  meets_min_order: boolean;
}

export interface AddCartItemInput {
  restaurant_id: string;
  menu_item_id: string;
  quantity: number;
  customizations: { customization_id: string; option_id: string }[];
  notes?: string;
}

export const cartApi = {
  get: async () => {
    const { data } = await apiClient.get<CartResponse | null>("/cart");
    return data;
  },
  add: async (input: AddCartItemInput) => {
    const { data } = await apiClient.post<CartResponse>("/cart/items", input);
    return data;
  },
  update: async (
    itemId: string,
    payload: { quantity?: number; notes?: string },
  ) => {
    const { data } = await apiClient.patch<CartResponse>(
      `/cart/items/${itemId}`,
      payload,
    );
    return data;
  },
  remove: async (itemId: string) => {
    const { data } = await apiClient.delete<CartResponse>(`/cart/items/${itemId}`);
    return data;
  },
  clear: async () => {
    await apiClient.delete("/cart");
  },
};

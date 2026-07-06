import { apiClient } from "./client";

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

export type PaymentStatus =
  | "pending"
  | "succeeded"
  | "failed"
  | "canceled"
  | "refunded"
  | "partially_refunded";

export interface OrderItem {
  id: string;
  menu_item_id: string | null;
  name: string;
  description: string | null;
  price_cents: number;
  quantity: number;
  customizations: unknown;
  notes: string | null;
  status: string;
  line_total_cents: number;
}

export interface Order {
  id: string;
  customer_id: string;
  restaurant_id: string;
  driver_id: string | null;
  status: OrderStatus;
  payment_status: PaymentStatus;
  snapshot: unknown;
  subtotal_cents: number;
  delivery_fee_cents: number;
  tax_cents: number;
  tip_cents: number;
  discount_cents: number;
  total_cents: number;
  currency: string;
  delivery_address: unknown;
  notes: string | null;
  placed_at: string;
  accepted_at: string | null;
  preparing_at: string | null;
  ready_at: string | null;
  picked_up_at: string | null;
  delivered_at: string | null;
  canceled_at: string | null;
  cancellation_reason: string | null;
  estimated_delivery_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface OrderDto extends Order {
  items: OrderItem[];
  restaurant_name: string;
}

export interface CreateOrderInput {
  address_id: string;
  payment_method_id?: string;
  promo_code?: string;
  tip_cents?: number;
  loyalty_points_to_redeem?: number;
  notes?: string;
}

export interface PaymentResult {
  intent_id: string;
  provider_intent_id: string | null;
  client_secret: string | null;
  status: string;
  amount_cents: number;
  currency: string;
  mock_mode: boolean;
}

export interface CreateOrderResponse {
  order: OrderDto;
  payment: PaymentResult;
}

export const ordersApi = {
  create: async (input: CreateOrderInput) => {
    const { data } = await apiClient.post<CreateOrderResponse>("/orders", input);
    return data;
  },
  list: async (page = 1, pageSize = 20) => {
    const { data } = await apiClient.get<OrderDto[]>("/orders", {
      params: { page, page_size: pageSize },
    });
    return data;
  },
  byId: async (id: string) => {
    const { data } = await apiClient.get<OrderDto>(`/orders/${id}`);
    return data;
  },
  cancel: async (id: string, reason: string) => {
    const { data } = await apiClient.post<OrderDto>(`/orders/${id}/cancel`, { reason });
    return data;
  },
};

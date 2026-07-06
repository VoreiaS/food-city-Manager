import { apiClient } from "./client";
import type { MenuResponse, OrderDto } from "@/types";
import type { OrderDto as OrderDtoType } from "./orders.api";

export type RestaurantOrderDto = OrderDtoType;

export type RestaurantStatus =
  | "pending_verification"
  | "active"
  | "paused"
  | "closing"
  | "closed";

export interface RestaurantProfile {
  id: string;
  name: string;
  slug: string;
  description?: string;
  cuisine_types: string[];
  price_range: number;
  logo_url: string | null;
  cover_url: string | null;
  delivery_fee_cents: number;
  min_order_cents: number;
  delivery_radius_m: number;
  status: RestaurantStatus;
  hours_json: Record<string, unknown>;
  rating_avg: number | null;
  rating_count: number;
}

export interface CreateItemInput {
  category_id: string;
  name: string;
  description?: string;
  price_cents: number;
  image_url?: string;
  is_veg?: boolean;
  spice_level?: number;
  allergens?: string[];
  track_stock?: boolean;
  stock_count?: number;
  sort_order?: number;
}

export interface UpdateItemInput {
  name?: string;
  description?: string;
  price_cents?: number;
  image_url?: string;
  is_veg?: boolean;
  spice_level?: number;
  stock_count?: number;
  status?: "available" | "out_of_stock" | "hidden";
  sort_order?: number;
}

export const restaurantApi = {
  orders: async (page = 1, pageSize = 50) => {
    const { data } = await apiClient.get<RestaurantOrderDto[]>("/restaurant/orders", {
      params: { page, page_size: pageSize },
    });
    return data;
  },
  acceptOrder: async (id: string) => {
    const { data } = await apiClient.post<OrderDto>(`/restaurant/orders/${id}/accept`);
    return data;
  },
  rejectOrder: async (id: string, reason: string) => {
    const { data } = await apiClient.post<OrderDto>(`/restaurant/orders/${id}/reject`, { reason });
    return data;
  },
  markPreparing: async (id: string) => {
    const { data } = await apiClient.post<OrderDto>(`/restaurant/orders/${id}/preparing`);
    return data;
  },
  markReady: async (id: string) => {
    const { data } = await apiClient.post<OrderDto>(`/restaurant/orders/${id}/ready`);
    return data;
  },
  menu: async () => {
    const { data } = await apiClient.get<MenuResponse>("/restaurant/menu");
    return data;
  },
  createItem: async (input: CreateItemInput) => {
    const { data } = await apiClient.post("/restaurant/menu", input);
    return data;
  },
  updateItem: async (id: string, input: UpdateItemInput) => {
    const { data } = await apiClient.patch(`/restaurant/menu/items/${id}`, input);
    return data;
  },
  deleteItem: async (id: string) => {
    await apiClient.delete(`/restaurant/menu/items/${id}`);
  },
  uploadItemPhoto: async (id: string, file: File) => {
    const form = new FormData();
    form.append("file", file);
    const { data } = await apiClient.post<{ image_url: string; size: number }>(
      `/restaurant/menu/items/${id}/photo`,
      form,
      { headers: { "Content-Type": "multipart/form-data" } },
    );
    return data;
  },
  createCategory: async (name: string, sortOrder = 0) => {
    const { data } = await apiClient.post("/restaurant/menu/categories", {
      name,
      sort_order: sortOrder,
    });
    return data;
  },
  deleteCategory: async (id: string) => {
    await apiClient.delete(`/restaurant/menu/categories/${id}`);
  },
  profile: async () => {
    const { data } = await apiClient.get<RestaurantProfile>("/restaurant/profile");
    return data;
  },
  updateProfile: async (input: Partial<RestaurantProfile>) => {
    const { data } = await apiClient.patch<RestaurantProfile>("/restaurant/profile", input);
    return data;
  },
  updateStatus: async (status: RestaurantStatus) => {
    const { data } = await apiClient.post<RestaurantProfile>("/restaurant/status", { status });
    return data;
  },
};

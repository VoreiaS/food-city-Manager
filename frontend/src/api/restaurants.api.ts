import { apiClient } from "./client";
import type { Restaurant, MenuResponse } from "@/types";

export interface RestaurantCard {
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
  rating_avg: number | null;
  rating_count: number;
  status: Restaurant["status"];
  is_open: boolean;
  distance_m: number | null;
  delivery_eta_min: number | null;
}

export interface RestaurantListResponse {
  data: RestaurantCard[];
  page: number;
  page_size: number;
  total: number;
}

export interface RestaurantDetailResponse extends Restaurant {
  is_open: boolean;
}

export interface RestaurantQuery {
  lat?: number;
  lng?: number;
  radius_m?: number;
  cuisine?: string;
  price_range?: number;
  veg_only?: boolean;
  rating_min?: number;
  sort?: "distance" | "rating" | "eta" | "promos";
  q?: string;
  page?: number;
  page_size?: number;
}

export const restaurantsApi = {
  list: async (query: RestaurantQuery = {}) => {
    const { data } = await apiClient.get<RestaurantListResponse>("/restaurants", {
      params: query,
    });
    return data;
  },

  listCuisines: async () => {
    const { data } = await apiClient.get<string[]>("/restaurants/cuisines");
    return data;
  },

  byId: async (id: string) => {
    const { data } = await apiClient.get<RestaurantDetailResponse>(`/restaurants/${id}`);
    return data;
  },

  bySlug: async (slug: string) => {
    const { data } = await apiClient.get<RestaurantDetailResponse>(
      `/restaurants/by-slug/${slug}`,
    );
    return data;
  },

  menu: async (restaurantId: string) => {
    const { data } = await apiClient.get<MenuResponse>(`/restaurants/${restaurantId}/menu`);
    return data;
  },
};

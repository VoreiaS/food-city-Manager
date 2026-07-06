import { apiClient } from "./client";

export interface LiveOrder {
  id: string;
  restaurant_name: string;
  status: string;
  total_cents: number;
  placed_at: string;
  driver_id: string | null;
}

export interface LiveDriver {
  id: string;
  user_id: string;
  status: string;
  current_lat: number | null;
  current_lng: number | null;
  current_order_id: string | null;
  rating_avg: number | null;
  total_deliveries: number;
}

export interface LiveRestaurant {
  id: string;
  name: string;
  status: string;
  rating_avg: number | null;
  rating_count: number;
}

export interface AnalyticsSummary {
  total_orders: number;
  active_orders: number;
  delivered_orders: number;
  canceled_orders: number;
  total_customers: number;
  total_restaurants: number;
  total_drivers: number;
  gmv_cents: number;
  avg_order_value_cents: number;
}

export const adminApi = {
  liveOrders: async () => {
    const { data } = await apiClient.get<LiveOrder[]>("/admin/live/orders");
    return data;
  },
  liveDrivers: async () => {
    const { data } = await apiClient.get<LiveDriver[]>("/admin/live/drivers");
    return data;
  },
  liveRestaurants: async () => {
    const { data } = await apiClient.get<LiveRestaurant[]>("/admin/live/restaurants");
    return data;
  },
  reassignOrder: async (orderId: string, driverId: string) => {
    const { data } = await apiClient.post(`/admin/orders/${orderId}/reassign`, {
      driver_id: driverId,
    });
    return data;
  },
  setRestaurantStatus: async (restaurantId: string, status: string) => {
    const { data } = await apiClient.post(`/admin/restaurants/${restaurantId}/status`, { status });
    return data;
  },
  analytics: async () => {
    const { data } = await apiClient.get<AnalyticsSummary>("/admin/analytics/summary");
    return data;
  },
};

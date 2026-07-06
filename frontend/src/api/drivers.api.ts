import { apiClient } from "./client";
import type { OrderDto } from "./orders.api";

export type DriverStatus =
  | "offline"
  | "available"
  | "assigned"
  | "en_route"
  | "at_restaurant"
  | "picked_up"
  | "delivering"
  | "delivered";

export interface Driver {
  id: string;
  user_id: string;
  vehicle_type: string;
  license_plate: string | null;
  current_lat: number | null;
  current_lng: number | null;
  status: DriverStatus;
  current_order_id: string | null;
  rating_avg: number | null;
  rating_count: number;
  acceptance_rate: number;
  total_deliveries: number;
}

// For driver app, "active orders" are orders that need a driver (status=ready, no driver)
// or assigned to this driver.
export interface DriverOrderOffer {
  order: OrderDto;
  distance_m: number | null;
  payout_cents: number;
}

export const driversApi = {
  me: async () => {
    const { data } = await apiClient.get<Driver>("/drivers/me");
    return data;
  },
  goOnline: async (vehicle_type?: string) => {
    const { data } = await apiClient.post<Driver>("/drivers/me/online", { vehicle_type });
    return data;
  },
  goOffline: async () => {
    const { data } = await apiClient.post<Driver>("/drivers/me/offline");
    return data;
  },
  updateLocation: async (lat: number, lng: number, heading?: number, speed_kph?: number) => {
    await apiClient.post("/drivers/me/location", { lat, lng, heading, speed_kph });
  },
  acceptOrder: async (orderId: string) => {
    const { data } = await apiClient.post<OrderDto>(`/drivers/orders/${orderId}/accept`);
    return data;
  },
  pickupOrder: async (orderId: string) => {
    const { data } = await apiClient.post<OrderDto>(`/drivers/orders/${orderId}/pickup`);
    return data;
  },
  deliverOrder: async (orderId: string) => {
    const { data } = await apiClient.post<OrderDto>(`/drivers/orders/${orderId}/deliver`);
    return data;
  },
};

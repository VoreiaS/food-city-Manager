import { apiClient } from "./client";

export interface Address {
  id: string;
  user_id: string;
  label: string;
  line1: string;
  line2: string | null;
  city: string;
  postal_code: string | null;
  lat: number;
  lng: number;
  formatted_address: string;
  is_default: boolean;
}

export interface NewAddressInput {
  label: string;
  line1: string;
  line2?: string;
  city: string;
  postal_code?: string;
  lat: number;
  lng: number;
  formatted_address: string;
  is_default?: boolean;
}

export const addressesApi = {
  list: async () => {
    const { data } = await apiClient.get<Address[]>("/addresses");
    return data;
  },
  create: async (input: NewAddressInput) => {
    const { data } = await apiClient.post<Address>("/addresses", input);
    return data;
  },
  remove: async (id: string) => {
    await apiClient.delete(`/addresses/${id}`);
  },
};

import { apiClient } from "./client";

export type LoyaltyTier = "silver" | "gold" | "platinum";

export interface LoyaltyAccount {
  points_balance: number;
  tier: LoyaltyTier;
  lifetime_points: number;
  next_tier_points: number;
  tier_benefits: string[];
}

export interface LoyaltyTransaction {
  id: string;
  account_id: string;
  points_delta: number;
  reason: string;
  order_id: string | null;
  created_at: string;
}

export const loyaltyApi = {
  me: async () => {
    const { data } = await apiClient.get<LoyaltyAccount>("/loyalty/me");
    return data;
  },
  transactions: async () => {
    const { data } = await apiClient.get<LoyaltyTransaction[]>("/loyalty/me/transactions");
    return data;
  },
};

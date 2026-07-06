import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { loyaltyApi, type LoyaltyAccount, type LoyaltyTransaction } from "@/api/loyalty.api";

export function useLoyalty() {
  return useQuery({
    queryKey: ["loyalty", "me"],
    queryFn: () => loyaltyApi.me(),
    staleTime: 60_000,
  });
}

export function useLoyaltyTransactions() {
  return useQuery({
    queryKey: ["loyalty", "transactions"],
    queryFn: () => loyaltyApi.transactions(),
    staleTime: 30_000,
  });
}

export type { LoyaltyAccount, LoyaltyTransaction };

// disambiguate unused import for useMutation/useQueryClient (kept for future mutations)
void useMutation;
void useQueryClient;
void toast;

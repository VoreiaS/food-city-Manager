import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { adminApi, type AnalyticsSummary } from "@/api/admin.api";
import { disputesApi, type Dispute } from "@/api/disputes.api";

export function useLiveOrders() {
  return useQuery({
    queryKey: ["admin", "live", "orders"],
    queryFn: () => adminApi.liveOrders(),
    refetchInterval: 10_000,
    staleTime: 5_000,
  });
}

export function useLiveDrivers() {
  return useQuery({
    queryKey: ["admin", "live", "drivers"],
    queryFn: () => adminApi.liveDrivers(),
    refetchInterval: 15_000,
  });
}

export function useLiveRestaurants() {
  return useQuery({
    queryKey: ["admin", "live", "restaurants"],
    queryFn: () => adminApi.liveRestaurants(),
    staleTime: 30_000,
  });
}

export function useAnalytics() {
  return useQuery({
    queryKey: ["admin", "analytics"],
    queryFn: () => adminApi.analytics(),
    staleTime: 60_000,
  });
}

export function useOpenDisputes() {
  return useQuery({
    queryKey: ["admin", "disputes"],
    queryFn: () => disputesApi.listOpen(),
    refetchInterval: 30_000,
  });
}

export function useResolveDispute() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      resolution,
      amount_cents,
      notes,
    }: {
      id: string;
      resolution: "full_refund" | "partial_refund" | "reject";
      amount_cents?: number;
      notes?: string;
    }) => disputesApi.resolve(id, { resolution, amount_cents, notes }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["admin", "disputes"] });
      toast.success("Dispute resolved");
    },
    onError: () => toast.error("Failed to resolve dispute"),
  });
}

export type { AnalyticsSummary, Dispute };

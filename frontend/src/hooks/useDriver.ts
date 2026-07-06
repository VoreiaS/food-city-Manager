import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { driversApi, type Driver } from "@/api/drivers.api";

export function useDriverProfile() {
  return useQuery({
    queryKey: ["driver", "me"],
    queryFn: () => driversApi.me(),
    staleTime: 30_000,
    retry: false, // 404 if not yet a driver
  });
}

export function useGoOnline() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (vehicle_type?: string) => driversApi.goOnline(vehicle_type),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["driver", "me"] });
      toast.success("You're online — waiting for orders");
    },
    onError: () => toast.error("Failed to go online"),
  });
}

export function useGoOffline() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => driversApi.goOffline(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["driver", "me"] });
      toast.success("You're offline");
    },
    onError: () => toast.error("Failed to go offline"),
  });
}

export function useAcceptOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (orderId: string) => driversApi.acceptOrder(orderId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["driver", "me"] });
      toast.success("Order accepted!");
    },
    onError: () => toast.error("Failed to accept order"),
  });
}

export function usePickupOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (orderId: string) => driversApi.pickupOrder(orderId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["driver", "me"] });
      toast.success("Marked as picked up");
    },
    onError: () => toast.error("Failed to update status"),
  });
}

export function useDeliverOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (orderId: string) => driversApi.deliverOrder(orderId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["driver", "me"] });
      toast.success("Order delivered! 🎉");
    },
    onError: () => toast.error("Failed to mark delivered"),
  });
}

export type { Driver };

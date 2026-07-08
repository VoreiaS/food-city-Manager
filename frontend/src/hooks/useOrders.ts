import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ordersApi, type CreateOrderInput, type OrderDto } from "@/api/orders.api";

const SAFE_DEFAULTS = {
  retry: false,
  refetchOnWindowFocus: false,
};

export function useOrders(page = 1) {
  return useQuery({
    queryKey: ["orders", page],
    queryFn: () => ordersApi.list(page),
    staleTime: 30_000,
    ...SAFE_DEFAULTS,
  });
}

export function useOrder(id: string | undefined) {
  return useQuery({
    queryKey: ["order", id],
    queryFn: () => ordersApi.byId(id as string),
    enabled: !!id,
    ...SAFE_DEFAULTS,
    refetchInterval: (q) => {
      const status = q.state.data?.status;
      if (
        status &&
        !["delivered", "canceled", "auto_rejected"].includes(status)
      ) {
        return 5000;
      }
      return false;
    },
  });
}

export function usePlaceOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateOrderInput) => ordersApi.create(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["orders"] });
      qc.invalidateQueries({ queryKey: ["cart"] });
    },
  });
}

export function useCancelOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      ordersApi.cancel(id, reason),
    onSuccess: (_data, vars) => {
      qc.invalidateQueries({ queryKey: ["orders"] });
      qc.invalidateQueries({ queryKey: ["order", vars.id] });
    },
  });
}

export type { OrderDto };

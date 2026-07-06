import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { addressesApi, type NewAddressInput, type Address } from "@/api/addresses.api";

export function useAddresses() {
  return useQuery({
    queryKey: ["addresses"],
    queryFn: () => addressesApi.list(),
    staleTime: 60_000,
  });
}

export function useCreateAddress() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: NewAddressInput) => addressesApi.create(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["addresses"] }),
  });
}

export function useDeleteAddress() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => addressesApi.remove(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["addresses"] }),
  });
}

export type { Address, NewAddressInput };

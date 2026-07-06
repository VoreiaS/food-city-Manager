import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  restaurantApi,
  type RestaurantProfile,
  type RestaurantStatus,
  type CreateItemInput,
  type UpdateItemInput,
} from "@/api/restaurant.api";
import type { MenuResponse } from "@/types";
export function useRestaurantProfile() {
  return useQuery({
    queryKey: ["restaurant", "profile"],
    queryFn: () => restaurantApi.profile(),
    staleTime: 60_000,
  });
}

export function useRestaurantOrders() {
  return useQuery({
    queryKey: ["restaurant", "orders"],
    queryFn: () => restaurantApi.orders(1, 50),
    refetchInterval: 10_000, // poll every 10s for new orders
    staleTime: 5_000,
  });
}

export function useRestaurantMenu() {
  return useQuery({
    queryKey: ["restaurant", "menu"],
    queryFn: () => restaurantApi.menu(),
    staleTime: 60_000,
  });
}

export function useAcceptOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restaurantApi.acceptOrder(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "orders"] });
      toast.success("Order accepted");
    },
    onError: () => toast.error("Failed to accept order"),
  });
}

export function useRejectOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      restaurantApi.rejectOrder(id, reason),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "orders"] });
      toast.success("Order rejected");
    },
    onError: () => toast.error("Failed to reject order"),
  });
}

export function useMarkPreparing() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restaurantApi.markPreparing(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["restaurant", "orders"] }),
    onError: () => toast.error("Failed to update status"),
  });
}

export function useMarkReady() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restaurantApi.markReady(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["restaurant", "orders"] }),
    onError: () => toast.error("Failed to update status"),
  });
}

export function useUpdateRestaurantStatus() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (status: RestaurantStatus) => restaurantApi.updateStatus(status),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "profile"] });
      toast.success("Status updated");
    },
    onError: () => toast.error("Failed to update status"),
  });
}

export function useCreateMenuItem() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateItemInput) => restaurantApi.createItem(input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "menu"] });
      toast.success("Item created");
    },
    onError: () => toast.error("Failed to create item"),
  });
}

export function useUpdateMenuItem() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateItemInput }) =>
      restaurantApi.updateItem(id, input),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "menu"] });
      toast.success("Item updated");
    },
    onError: () => toast.error("Failed to update item"),
  });
}

export function useDeleteMenuItem() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restaurantApi.deleteItem(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "menu"] });
      toast.success("Item deleted");
    },
    onError: () => toast.error("Failed to delete item"),
  });
}

export function useUploadItemPhoto() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, file }: { id: string; file: File }) =>
      restaurantApi.uploadItemPhoto(id, file),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "menu"] });
      toast.success("Photo uploaded");
    },
    onError: () => toast.error("Failed to upload photo"),
  });
}

export function useCreateCategory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => restaurantApi.createCategory(name),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "menu"] });
      toast.success("Category added");
    },
    onError: () => toast.error("Failed to add category"),
  });
}

export function useDeleteCategory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restaurantApi.deleteCategory(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["restaurant", "menu"] });
      toast.success("Category deleted");
    },
    onError: () => toast.error("Failed to delete category"),
  });
}

export type { RestaurantProfile, RestaurantStatus, MenuResponse, CreateItemInput, UpdateItemInput };

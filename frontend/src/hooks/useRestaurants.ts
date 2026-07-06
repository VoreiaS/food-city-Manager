import { useQuery } from "@tanstack/react-query";
import { restaurantsApi, type RestaurantQuery } from "@/api/restaurants.api";

// Default location (Colombo, Sri Lanka) — used if user hasn't shared location.
export const DEFAULT_LAT = 6.9271;
export const DEFAULT_LNG = 79.8612;

export function useRestaurants(query: RestaurantQuery = {}) {
  return useQuery({
    queryKey: ["restaurants", query],
    queryFn: () =>
      restaurantsApi.list({
        lat: DEFAULT_LAT,
        lng: DEFAULT_LNG,
        radius_m: 10000,
        ...query,
      }),
    staleTime: 60_000,
  });
}

export function useCuisines() {
  return useQuery({
    queryKey: ["cuisines"],
    queryFn: () => restaurantsApi.listCuisines(),
    staleTime: 5 * 60_000,
  });
}

export function useRestaurant(id: string | undefined) {
  return useQuery({
    queryKey: ["restaurant", id],
    queryFn: () => restaurantsApi.byId(id as string),
    enabled: !!id,
  });
}

export function useRestaurantBySlug(slug: string | undefined) {
  return useQuery({
    queryKey: ["restaurant", "slug", slug],
    queryFn: () => restaurantsApi.bySlug(slug as string),
    enabled: !!slug,
  });
}

export function useMenu(restaurantId: string | undefined) {
  return useQuery({
    queryKey: ["menu", restaurantId],
    queryFn: () => restaurantsApi.menu(restaurantId as string),
    enabled: !!restaurantId,
    staleTime: 60_000,
  });
}

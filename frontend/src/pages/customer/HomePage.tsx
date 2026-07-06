import { useMemo, useState } from "react";
import { Search, ShoppingBag } from "lucide-react";
import { useRestaurants, useCuisines } from "@/hooks/useRestaurants";
import { useCartStore } from "@/store/cartStore";
import { RestaurantCardItem } from "@/components/restaurant/RestaurantCard";
import { CartDrawer } from "@/components/cart/CartDrawer";
import { Button } from "@/components/ui/Button";
import { RestaurantListSkeleton } from "@/components/common/Skeleton";
import { clsx } from "clsx";

export function HomePage() {
  const openDrawer = useCartStore((s) => s.openDrawer);
  const cart = useCartStore((s) => s.cart);

  const [search, setSearch] = useState("");
  const [cuisineFilter, setCuisineFilter] = useState<string | undefined>();
  const [sort, setSort] = useState<"distance" | "rating" | "eta">("distance");

  const query = useMemo(
    () => ({
      q: search || undefined,
      cuisine: cuisineFilter,
      sort,
      page_size: 30,
    }),
    [search, cuisineFilter, sort],
  );

  const { data, isLoading } = useRestaurants(query);
  const { data: cuisines } = useCuisines();

  return (
    <div>
      {/* Hero search */}
      <section className="bg-gradient-to-br from-brand-50 via-white to-amber-50">
        <div className="mx-auto max-w-7xl px-4 py-10">
          <h1 className="font-display text-3xl md:text-4xl font-bold">
            Your city's best food, <span className="text-brand-500">delivered.</span>
          </h1>
          <p className="mt-2 text-gray-600">
            Discover restaurants near you, order in minutes, and track live.
          </p>
          <div className="mt-4 flex gap-2">
            <div className="relative flex-1">
              <Search
                size={18}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
              />
              <input
                className="input pl-10"
                placeholder="Search restaurants or cuisines…"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
            <Button onClick={openDrawer} variant="secondary" className="relative">
              <ShoppingBag size={16} />
              Cart
              {cart && cart.items.length > 0 && (
                <span className="absolute -top-1 -right-1 grid h-5 min-w-5 place-items-center rounded-full bg-brand-500 px-1 text-xs text-white">
                  {cart.items.length}
                </span>
              )}
            </Button>
          </div>

          {/* Cuisine chips */}
          {cuisines && cuisines.length > 0 && (
            <div className="mt-4 flex flex-wrap gap-2">
              <button
                onClick={() => setCuisineFilter(undefined)}
                className={clsx(
                  "rounded-full px-3 py-1 text-xs font-medium transition",
                  !cuisineFilter
                    ? "bg-brand-500 text-white"
                    : "bg-white text-gray-700 ring-1 ring-gray-200 hover:bg-gray-50",
                )}
              >
                All
              </button>
              {cuisines.map((c) => (
                <button
                  key={c}
                  onClick={() => setCuisineFilter(c)}
                  className={clsx(
                    "rounded-full px-3 py-1 text-xs font-medium capitalize transition",
                    cuisineFilter === c
                      ? "bg-brand-500 text-white"
                      : "bg-white text-gray-700 ring-1 ring-gray-200 hover:bg-gray-50",
                  )}
                >
                  {c}
                </button>
              ))}
            </div>
          )}
        </div>
      </section>

      {/* Restaurant list */}
      <section className="mx-auto max-w-7xl px-4 py-8">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="font-display text-xl font-semibold">
            {data ? `${data.total} restaurant${data.total === 1 ? "" : "s"}` : "Restaurants"}
          </h2>
          <select
            value={sort}
            onChange={(e) => setSort(e.target.value as typeof sort)}
            className="input max-w-[180px]"
          >
            <option value="distance">Nearest</option>
            <option value="rating">Top rated</option>
            <option value="eta">Fastest</option>
          </select>
        </div>

        {isLoading ? (
          <RestaurantListSkeleton count={8} />
        ) : data && data.data.length > 0 ? (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {data.data.map((r) => (
              <RestaurantCardItem key={r.id} restaurant={r} />
            ))}
          </div>
        ) : (
          <div className="card p-8 text-center text-gray-500">
            <p>No restaurants found. Try a different search or cuisine.</p>
          </div>
        )}
      </section>

      <CartDrawer />
    </div>
  );
}

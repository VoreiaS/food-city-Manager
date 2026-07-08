import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Star, Clock, Bike, ChevronLeft, Leaf, Flame, WifiOff } from "lucide-react";
import { useRestaurant, useMenu } from "@/hooks/useRestaurants";
import { CartDrawer } from "@/components/cart/CartDrawer";
import { AddItemModal } from "@/components/cart/AddItemModal";
import { clsx } from "clsx";
import type { MenuItem } from "@/types";

interface Props {
  restaurantId: string;
}

export function RestaurantDetail({ restaurantId }: Props) {
  const { data: restaurant, isLoading: rLoading, isError: rError } = useRestaurant(restaurantId);
  const { data: menu, isLoading: mLoading, isError: mError } = useMenu(restaurantId);
  const [selectedItem, setSelectedItem] = useState<MenuItem | null>(null);

  if (rLoading || mLoading) {
    return <div className="mx-auto max-w-4xl px-4 py-12 text-center text-gray-500">Loading…</div>;
  }

  if (rError || mError) {
    return (
      <div className="mx-auto max-w-4xl px-4 py-12 text-center">
        <WifiOff size={48} className="mx-auto text-gray-300" />
        <h2 className="mt-3 font-semibold">Can't load restaurant</h2>
        <p className="mt-1 text-sm text-gray-500">
          The backend API isn't available. Make sure the server is running.
        </p>
      </div>
    );
  }

  if (!restaurant || !menu) {
    return <div className="mx-auto max-w-4xl px-4 py-12 text-center text-gray-500">Not found.</div>;
  }

  // Safe access to categories — default to empty array
  const categories = menu.categories ?? [];

  const r = restaurant;

  return (
    <div>
      {/* Cover */}
      <div className="relative h-48 bg-gradient-to-br from-brand-200 to-amber-200">
        <BackButton />
        {r.cover_url && <img src={r.cover_url} alt="" className="h-full w-full object-cover" />}
      </div>

      <div className="mx-auto max-w-4xl px-4">
        {/* Header card */}
        <div className="card -mt-12 relative p-5">
          <div className="flex items-start justify-between gap-4">
            <div>
              <h1 className="font-display text-2xl font-bold">{r.name}</h1>
              <p className="mt-1 text-sm text-gray-600">{r.cuisine_types.join(" · ")}</p>
              <p className="mt-2 text-sm text-gray-700">{r.description}</p>
            </div>
            <div className="shrink-0 text-right">
              {r.rating_avg && (
                <div className="flex items-center gap-1 text-amber-700">
                  <Star size={16} fill="currentColor" />
                  <span className="font-semibold">{r.rating_avg.toFixed(1)}</span>
                  <span className="text-xs text-gray-500">({r.rating_count})</span>
                </div>
              )}
              <div className="mt-1 flex items-center gap-1 text-xs text-gray-500">
                <Clock size={12} /> {r.is_open ? "Open now" : "Closed"}
              </div>
            </div>
          </div>

          <div className="mt-4 flex flex-wrap gap-3 border-t border-gray-100 pt-4 text-sm text-gray-600">
            <span className="flex items-center gap-1.5">
              <Bike size={14} /> ${(r.delivery_fee_cents / 100).toFixed(2)} delivery
            </span>
            <span className="flex items-center gap-1.5">
              <Clock size={14} /> 25-35 min
            </span>
            <span>Min order ${(r.min_order_cents / 100).toFixed(2)}</span>
            <span>{"$".repeat(r.price_range)}</span>
          </div>
        </div>

        {/* Closed banner */}
        {!r.is_open && (
          <div className="mt-4 rounded-lg bg-amber-50 border border-amber-200 p-3 text-sm text-amber-800 flex items-center gap-2">
            <Clock size={16} />
            This restaurant is currently closed. You can browse the menu but can't place an order right now.
          </div>
        )}

        {/* Menu */}
        <div className="mt-8 space-y-8 pb-24">
          {categories.map((cat) => {
            const items = cat.items ?? [];
            if (items.length === 0) return null;
            return (
              <section key={cat.id}>
                <h2 className="font-display text-lg font-semibold mb-3">{cat.name}</h2>
                <div className="space-y-2">
                  {items.map((item) => (
                    <button
                      key={item.id}
                      onClick={() => item.in_stock && r.is_open && setSelectedItem(item)}
                      disabled={!item.in_stock || !r.is_open}
                      className={clsx(
                        "card w-full p-4 text-left transition",
                        item.in_stock && r.is_open
                          ? "hover:shadow-md hover:border-brand-200"
                          : "opacity-50",
                      )}
                    >
                      <div className="flex gap-4">
                        <div className="flex-1">
                          <div className="flex items-center gap-2">
                            {item.is_veg && <Leaf size={14} className="text-green-600" />}
                            {item.spice_level > 0 && (
                              <span className="flex items-center text-red-600">
                                {Array.from({ length: item.spice_level }).map((_, i) => (
                                  <Flame key={i} size={10} />
                                ))}
                              </span>
                            )}
                            <h3 className="font-semibold">{item.name}</h3>
                          </div>
                          <p className="mt-1 text-sm text-gray-600 line-clamp-2">{item.description}</p>
                          <p className="mt-2 text-sm font-medium text-brand-700">
                            ${(item.price_cents / 100).toFixed(2)}
                          </p>
                        </div>
                        {item.image_url && (
                          <img
                            src={item.image_url}
                            alt=""
                            className="h-20 w-20 shrink-0 rounded-lg object-cover"
                          />
                        )}
                      </div>
                    </button>
                  ))}
                </div>
              </section>
            );
          })}
        </div>
      </div>

      {selectedItem && (
        <AddItemModal
          item={selectedItem}
          restaurantId={restaurantId}
          onClose={() => setSelectedItem(null)}
        />
      )}
      <CartDrawer />
    </div>
  );
}

function BackButton() {
  const navigate = useNavigate();
  return (
    <button
      onClick={() => navigate(-1)}
      className="absolute top-3 left-3 grid h-9 w-9 place-items-center rounded-full bg-white/90 shadow-sm hover:bg-white"
      aria-label="Back"
    >
      <ChevronLeft size={18} />
    </button>
  );
}

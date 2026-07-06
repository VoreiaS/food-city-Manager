import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Bell, Package, ChevronRight } from "lucide-react";
import { useDriverProfile } from "@/hooks/useDriver";
import { apiClient } from "@/api/client";
import type { OrderDto } from "@/api/orders.api";
import { Button } from "@/components/ui/Button";

export function OrderOfferPage() {
  const { data: driver } = useDriverProfile();

  // Look up orders in 'ready' state with no driver assigned (potential offers)
  // In production this would be a dedicated endpoint with distance/payout info.
  const { data: availableOrders } = useQuery({
    queryKey: ["driver", "available-orders"],
    queryFn: async () => {
      // We don't have a public "available orders" endpoint for drivers,
      // so this is a placeholder. In production, the driver_match_loop
      // would auto-assign, and this page would show the current assignment.
      return [] as OrderDto[];
    },
    refetchInterval: 10_000,
  });

  // Show the driver's current active order if any
  const { data: activeOrder } = useQuery({
    queryKey: ["driver", "active-order", driver?.current_order_id],
    queryFn: async () => {
      if (!driver?.current_order_id) return null;
      const { data } = await apiClient.get<OrderDto>(
        `/orders/${driver.current_order_id}`,
      );
      return data;
    },
    enabled: !!driver?.current_order_id,
  });

  if (!driver) {
    return <div className="p-8 text-center text-gray-500">Loading…</div>;
  }

  return (
    <div className="mx-auto max-w-md px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-2">Order Offers</h1>
      <p className="text-sm text-gray-500 mb-6">
        New orders near you will appear here. Accept quickly — first come, first served.
      </p>

      {/* Active order */}
      {activeOrder && (
        <div className="card p-4 mb-6 border-l-4 border-blue-500">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-xs text-gray-500 uppercase">Currently assigned</div>
              <div className="font-semibold">{activeOrder.restaurant_name}</div>
              <div className="text-sm text-gray-600">
                Order #{activeOrder.id.slice(0, 8)} · ${(activeOrder.total_cents / 100).toFixed(2)}
              </div>
            </div>
            <Button size="sm" as-child>
              <Link to="/driver/active">Go <ChevronRight size={14} /></Link>
            </Button>
          </div>
        </div>
      )}

      {/* Available offers */}
      {!availableOrders || availableOrders.length === 0 ? (
        <div className="card p-8 text-center">
          <Bell size={48} className="mx-auto text-gray-300" />
          <h2 className="mt-3 font-semibold">No offers right now</h2>
          <p className="mt-1 text-sm text-gray-500">
            Stay online — offers arrive automatically when restaurants mark food ready.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {availableOrders.map((o) => (
            <div key={o.id} className="card p-4">
              <div className="flex items-start justify-between">
                <div>
                  <div className="font-semibold">{o.restaurant_name}</div>
                  <div className="text-sm text-gray-600">
                    <Package size={12} className="inline mr-1" />
                    {o.items.length} item{o.items.length === 1 ? "" : "s"} ·{" "}
                    ${(o.total_cents / 100).toFixed(2)}
                  </div>
                </div>
                <Button size="sm">Accept</Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* How it works */}
      <div className="mt-6 card p-4 text-sm text-gray-500">
        <h3 className="font-semibold text-gray-700 mb-1">How offers work</h3>
        <p>
          When a restaurant marks an order as ready, our system finds the nearest
          available driver and assigns automatically. Stay online and near busy
          areas to receive more offers.
        </p>
      </div>
    </div>
  );
}

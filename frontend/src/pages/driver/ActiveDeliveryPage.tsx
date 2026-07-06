import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Package, MapPin, Store, Navigation, CheckCircle2, Truck } from "lucide-react";
import { useDriverProfile, usePickupOrder, useDeliverOrder } from "@/hooks/useDriver";
import { apiClient } from "@/api/client";
import type { OrderDto } from "@/api/orders.api";
import { Button } from "@/components/ui/Button";

export function ActiveDeliveryPage() {
  const { data: driver } = useDriverProfile();
  const pickup = usePickupOrder();
  const deliver = useDeliverOrder();

  // Fetch the driver's current order.
  const { data: order } = useQuery({
    queryKey: ["driver", "active-order", driver?.current_order_id],
    queryFn: async () => {
      if (!driver?.current_order_id) return null;
      const { data } = await apiClient.get<OrderDto>(
        `/orders/${driver.current_order_id}`,
      );
      return data;
    },
    enabled: !!driver?.current_order_id,
    refetchInterval: 5_000,
  });

  if (!driver) {
    return (
      <div className="p-8 text-center text-gray-500">
        Loading driver profile…
      </div>
    );
  }

  if (!driver.current_order_id || !order) {
    return (
      <div className="mx-auto max-w-md px-4 py-12 text-center">
        <Package size={48} className="mx-auto text-gray-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">No active delivery</h1>
        <p className="mt-1 text-sm text-gray-500">
          When you accept an order, it'll show up here with pickup and delivery details.
        </p>
        <Button className="mt-4" variant="secondary" as-child>
          <Link to="/driver">Back to shift</Link>
        </Button>
      </div>
    );
  }

  const restaurant = order.restaurant_name;
  const items = order.items;

  return (
    <div className="mx-auto max-w-md px-4 py-6">
      <h1 className="font-display text-2xl font-bold">Active Delivery</h1>
      <p className="text-sm text-gray-500">Order #{order.id.slice(0, 8)}</p>

      {/* Status banner */}
      <div className="mt-4 rounded-lg bg-brand-50 p-3 text-center text-sm font-medium text-brand-700">
        Status: <span className="capitalize">{order.status.replace("_", " ")}</span>
      </div>

      {/* Pickup info */}
      <section className="mt-4 card p-4">
        <div className="flex items-start gap-3">
          <Store size={20} className="text-brand-500 mt-0.5" />
          <div className="flex-1">
            <div className="font-semibold">{restaurant}</div>
            <div className="text-xs text-gray-500">Pickup at restaurant</div>
          </div>
        </div>
      </section>

      {/* Order items */}
      <section className="mt-4 card p-4">
        <h2 className="font-semibold mb-2 flex items-center gap-2">
          <Package size={16} /> Items to pick up
        </h2>
        <ul className="space-y-1 text-sm">
          {items.map((i) => (
            <li key={i.id} className="flex justify-between">
              <span>
                <span className="text-gray-500">{i.quantity}×</span> {i.name}
              </span>
            </li>
          ))}
        </ul>
        {order.notes && (
          <p className="mt-3 text-xs italic text-gray-500 border-t border-gray-100 pt-2">
            "{order.notes}"
          </p>
        )}
      </section>

      {/* Drop-off info */}
      <section className="mt-4 card p-4">
        <div className="flex items-start gap-3">
          <MapPin size={20} className="text-brand-500 mt-0.5" />
          <div className="flex-1">
            <div className="font-semibold">Customer drop-off</div>
            <div className="text-xs text-gray-500">
              Address details available in delivery_address
            </div>
          </div>
        </div>
      </section>

      {/* Actions */}
      <div className="mt-6 space-y-2">
        {order.status === "ready" && (
          <Button
            className="w-full"
            size="lg"
            onClick={() => pickup.mutate(order.id)}
            disabled={pickup.isPending}
          >
            <Navigation size={16} /> Confirm pickup
          </Button>
        )}
        {order.status === "picked_up" && (
          <Button
            className="w-full"
            size="lg"
            onClick={() => {
              if (confirm("Confirm customer received the order?")) {
                deliver.mutate(order.id);
              }
            }}
            disabled={deliver.isPending}
          >
            <CheckCircle2 size={16} /> Mark delivered
          </Button>
        )}
        {order.status === "delivering" && (
          <div className="text-center text-sm text-gray-500">
            <Truck size={32} className="mx-auto mb-2 text-gray-400" />
            En route to customer…
          </div>
        )}
        {order.status === "delivered" && (
          <div className="text-center text-sm text-green-600">
            <CheckCircle2 size={32} className="mx-auto mb-2" />
            Delivered! Payout will appear in your earnings.
            <Button className="mt-3" variant="secondary" as-child>
              <Link to="/driver">Back to shift</Link>
            </Button>
          </div>
        )}
      </div>

      <p className="mt-4 text-xs text-gray-400 text-center">
        Photo proof + GPS verification wire up in Phase 9.
      </p>
    </div>
  );
}

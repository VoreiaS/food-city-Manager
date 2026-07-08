import { Link } from "react-router-dom";
import { Clock, MapPin, WifiOff } from "lucide-react";
import { useOrders } from "@/hooks/useOrders";
import type { OrderStatus } from "@/api/orders.api";
import { Button } from "@/components/ui/Button";

const statusColor: Record<OrderStatus, string> = {
  pending_accept: "bg-amber-100 text-amber-700",
  accepted: "bg-blue-100 text-blue-700",
  preparing: "bg-blue-100 text-blue-700",
  ready: "bg-indigo-100 text-indigo-700",
  picked_up: "bg-purple-100 text-purple-700",
  delivering: "bg-purple-100 text-purple-700",
  delivered: "bg-green-100 text-green-700",
  canceled: "bg-red-100 text-red-700",
  auto_rejected: "bg-red-100 text-red-700",
};

const statusLabel: Record<OrderStatus, string> = {
  pending_accept: "Pending",
  accepted: "Accepted",
  preparing: "Preparing",
  ready: "Ready",
  picked_up: "Picked up",
  delivering: "On the way",
  delivered: "Delivered",
  canceled: "Canceled",
  auto_rejected: "Auto-rejected",
};

export function OrdersPage() {
  const { data: orders, isLoading, isError } = useOrders(1);

  if (isLoading) {
    return <div className="mx-auto max-w-3xl px-4 py-12 text-center text-gray-500">Loading…</div>;
  }

  if (isError) {
    return (
      <div className="mx-auto max-w-md px-4 py-12 text-center">
        <WifiOff size={48} className="mx-auto text-gray-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">Can't load orders</h1>
        <p className="mt-1 text-sm text-gray-500">
          The backend API isn't available. Make sure the server is running.
        </p>
        <Button className="mt-4" as-child>
          <Link to="/">Browse restaurants</Link>
        </Button>
      </div>
    );
  }

  // Safe access — orders might be undefined
  const orderList = orders ?? [];

  if (orderList.length === 0) {
    return (
      <div className="mx-auto max-w-md px-4 py-12 text-center">
        <Clock size={48} className="mx-auto text-gray-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">No orders yet</h1>
        <p className="mt-1 text-sm text-gray-500">
          When you place an order, it'll show up here with live tracking.
        </p>
        <Button className="mt-4" as-child>
          <Link to="/">Browse restaurants</Link>
        </Button>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Your Orders</h1>
      <div className="space-y-3">
        {orderList.map((o) => (
          <Link
            key={o.id}
            to={`/orders/${o.id}/track`}
            className="card block p-4 transition hover:shadow-md"
          >
            <div className="flex items-start justify-between gap-3">
              <div>
                <div className="font-semibold">{o.restaurant_name}</div>
                <div className="mt-1 text-xs text-gray-500">
                  {new Date(o.placed_at).toLocaleString()}
                </div>
                <div className="mt-1 text-xs text-gray-600 flex items-center gap-1">
                  <MapPin size={12} />
                  {(o.items.length === 1 ? "1 item" : `${o.items.length} items`)} ·{" "}
                  ${(o.total_cents / 100).toFixed(2)}
                </div>
              </div>
              <span
                className={`rounded-full px-2.5 py-0.5 text-xs font-medium ${
                  statusColor[o.status]
                }`}
              >
                {statusLabel[o.status]}
              </span>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}

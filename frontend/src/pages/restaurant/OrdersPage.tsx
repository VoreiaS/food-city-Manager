import { useRestaurantOrders, useAcceptOrder, useRejectOrder, useMarkPreparing, useMarkReady } from "@/hooks/useRestaurant";
import { Button } from "@/components/ui/Button";
import { Clock, CheckCircle2, XCircle, ChefHat, Bell } from "lucide-react";
import { clsx } from "clsx";
import type { OrderStatus } from "@/api/orders.api";

const statusLabel: Record<OrderStatus, string> = {
  pending_accept: "Pending",
  accepted: "Accepted",
  preparing: "Preparing",
  ready: "Ready for pickup",
  picked_up: "Picked up",
  delivering: "On the way",
  delivered: "Delivered",
  canceled: "Canceled",
  auto_rejected: "Auto-rejected",
};

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

export function OrdersPage() {
  const { data: orders, isLoading } = useRestaurantOrders();
  const accept = useAcceptOrder();
  const reject = useRejectOrder();
  const preparing = useMarkPreparing();
  const ready = useMarkReady();

  if (isLoading) return <div className="p-8 text-center text-gray-500">Loading…</div>;

  if (!orders || orders.length === 0) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-12 text-center">
        <Bell size={48} className="mx-auto text-gray-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">No orders yet</h1>
        <p className="mt-1 text-sm text-gray-500">
          New orders will appear here in real time.
        </p>
      </div>
    );
  }

  // Sort: pending_accept first, then by placed_at
  const sorted = [...orders].sort((a, b) => {
    if (a.status === "pending_accept" && b.status !== "pending_accept") return -1;
    if (b.status === "pending_accept" && a.status !== "pending_accept") return 1;
    return new Date(b.placed_at).getTime() - new Date(a.placed_at).getTime();
  });

  const active = sorted.filter((o) =>
    ["pending_accept", "accepted", "preparing", "ready", "picked_up", "delivering"].includes(
      o.status,
    ),
  );

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Orders</h1>

      <div className="space-y-3">
        {active.map((o) => (
          <div key={o.id} className="card p-4">
            <div className="flex items-start justify-between gap-3">
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <span
                    className={clsx(
                      "rounded-full px-2.5 py-0.5 text-xs font-medium",
                      statusColor[o.status],
                    )}
                  >
                    {statusLabel[o.status]}
                  </span>
                  <span className="text-xs text-gray-500">
                    {new Date(o.placed_at).toLocaleTimeString()}
                  </span>
                </div>
                <div className="mt-2 text-sm font-medium">Order #{o.id.slice(0, 8)}</div>
                <ul className="mt-1 text-sm text-gray-600">
                  {o.items.map((i) => (
                    <li key={i.id}>
                      {i.quantity}× {i.name}
                      {i.notes && <span className="text-xs text-gray-400"> — {i.notes}</span>}
                    </li>
                  ))}
                </ul>
                <div className="mt-2 text-sm font-medium text-brand-700">
                  Total: ${(o.total_cents / 100).toFixed(2)}
                </div>
              </div>

              {/* Actions */}
              <div className="flex flex-col gap-1.5">
                {o.status === "pending_accept" && (
                  <>
                    <Button
                      size="sm"
                      onClick={() => accept.mutate(o.id)}
                      disabled={accept.isPending}
                    >
                      <CheckCircle2 size={14} /> Accept
                    </Button>
                    <Button
                      size="sm"
                      variant="secondary"
                      onClick={() => {
                        if (confirm("Reject this order? Customer will be refunded.")) {
                          reject.mutate({ id: o.id, reason: "restaurant_rejected" });
                        }
                      }}
                    >
                      <XCircle size={14} /> Reject
                    </Button>
                  </>
                )}
                {o.status === "accepted" && (
                  <Button
                    size="sm"
                    onClick={() => preparing.mutate(o.id)}
                    disabled={preparing.isPending}
                  >
                    <ChefHat size={14} /> Start preparing
                  </Button>
                )}
                {o.status === "preparing" && (
                  <Button
                    size="sm"
                    onClick={() => ready.mutate(o.id)}
                    disabled={ready.isPending}
                  >
                    <Bell size={14} /> Mark ready
                  </Button>
                )}
                {(o.status === "ready" ||
                  o.status === "picked_up" ||
                  o.status === "delivering") && (
                  <div className="text-xs text-gray-500">
                    <Clock size={12} className="inline" /> Awaiting driver
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      <h2 className="font-display text-lg font-semibold mt-8 mb-3">Past orders</h2>
      <div className="space-y-2">
        {sorted
          .filter((o) => !active.includes(o))
          .slice(0, 10)
          .map((o) => (
            <div key={o.id} className="card p-3 text-sm flex items-center justify-between">
              <span>Order #{o.id.slice(0, 8)} · {new Date(o.placed_at).toLocaleString()}</span>
              <span
                className={clsx(
                  "rounded-full px-2 py-0.5 text-xs font-medium",
                  statusColor[o.status],
                )}
              >
                {statusLabel[o.status]}
              </span>
            </div>
          ))}
      </div>
    </div>
  );
}

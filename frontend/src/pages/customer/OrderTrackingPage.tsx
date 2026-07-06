import { useEffect, useMemo } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { toast } from "sonner";
import { ChevronLeft, CheckCircle2, XCircle, Loader2, Wifi, WifiOff, Bike } from "lucide-react";
import { useOrder, useCancelOrder } from "@/hooks/useOrders";
import { useOrderTracking, useDriverLocation } from "@/hooks/useOrderTracking";
import { useRestaurant } from "@/hooks/useRestaurants";
import type { OrderStatus } from "@/api/orders.api";
import { Button } from "@/components/ui/Button";
import { LeafletMap, type MapPin } from "@/components/map/LeafletMap";
import { clsx } from "clsx";

const STEPS: OrderStatus[] = [
  "pending_accept",
  "accepted",
  "preparing",
  "ready",
  "picked_up",
  "delivering",
  "delivered",
];

const LABEL: Record<OrderStatus, string> = {
  pending_accept: "Order placed",
  accepted: "Restaurant accepted",
  preparing: "Preparing your food",
  ready: "Ready for pickup",
  picked_up: "Driver picked up",
  delivering: "On the way",
  delivered: "Delivered",
  canceled: "Canceled",
  auto_rejected: "Auto-rejected",
};

interface DeliveryAddress {
  lat?: number;
  lng?: number;
  formatted_address?: string;
  line1?: string;
  city?: string;
}

export function OrderTrackingPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { data: order, isLoading } = useOrder(id);
  const { data: restaurant } = useRestaurant(order?.restaurant_id);
  const cancelOrder = useCancelOrder();
  const { lastEvent, isConnected } = useOrderTracking(id);
  const { location: driverLocation, path: driverPath } = useDriverLocation(id);

  // Show toast on status changes
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.event_type === "order.canceled") {
      toast.error("Order was canceled");
    } else if (lastEvent.event_type === "order.delivered") {
      toast.success("Order delivered! Enjoy your meal.");
    } else if (lastEvent.event_type === "driver.assigned") {
      toast.info("A driver has been assigned to your order!");
    } else if (lastEvent.event_type?.startsWith("order.")) {
      toast.info(`Status: ${lastEvent.event_type.replace("order.", "")}`);
    }
  }, [lastEvent]);

  // Parse delivery address from order
  const deliveryAddress = useMemo<DeliveryAddress>(() => {
    if (!order?.delivery_address) return {};
    return order.delivery_address as DeliveryAddress;
  }, [order]);

  // Build map pins
  const pins: MapPin[] = useMemo(() => {
    const result: MapPin[] = [];
    if (restaurant) {
      result.push({
        lat: restaurant.lat,
        lng: restaurant.lng,
        label: restaurant.name,
        type: "restaurant",
      });
    }
    if (deliveryAddress.lat && deliveryAddress.lng) {
      result.push({
        lat: deliveryAddress.lat,
        lng: deliveryAddress.lng,
        label: "Your address",
        type: "customer",
      });
    }
    if (driverLocation) {
      result.push({
        lat: driverLocation.lat,
        lng: driverLocation.lng,
        label: "Driver",
        type: "driver",
      });
    }
    return result;
  }, [restaurant, deliveryAddress, driverLocation]);

  if (isLoading || !order) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-12 text-center text-gray-500">
        {isLoading ? "Loading…" : "Order not found."}
      </div>
    );
  }

  const canceled = order.status === "canceled" || order.status === "auto_rejected";
  const currentStepIdx = STEPS.indexOf(order.status);
  const showMap = !canceled && order.status !== "delivered" && pins.length > 0;
  const hasDriver = ["picked_up", "delivering"].includes(order.status) || !!driverLocation;

  const handleCancel = async () => {
    if (!id) return;
    if (!confirm("Cancel this order? Refund depends on preparation status.")) return;
    try {
      await cancelOrder.mutateAsync({ id, reason: "customer_canceled" });
      toast.success("Order canceled");
    } catch {
      toast.error("Failed to cancel");
    }
  };

  return (
    <div className="mx-auto max-w-2xl px-4 py-6">
      <button
        onClick={() => navigate(-1)}
        className="mb-3 flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
      >
        <ChevronLeft size={16} /> Back
      </button>

      <h1 className="font-display text-2xl font-bold">{order.restaurant_name}</h1>
      <p className="text-sm text-gray-500">
        Order #{order.id.slice(0, 8)} · {new Date(order.placed_at).toLocaleString()}
      </p>

      {/* Status banner */}
      <div
        className={clsx(
          "mt-4 rounded-lg p-4 text-sm font-medium",
          canceled
            ? "bg-red-50 text-red-700"
            : order.status === "delivered"
              ? "bg-green-50 text-green-700"
              : "bg-brand-50 text-brand-700",
        )}
      >
        {canceled ? (
          <span className="flex items-center gap-2">
            <XCircle size={16} /> {LABEL[order.status]}
            {order.cancellation_reason && ` — ${order.cancellation_reason}`}
          </span>
        ) : order.status === "delivered" ? (
          <span className="flex items-center gap-2">
            <CheckCircle2 size={16} /> {LABEL[order.status]}
          </span>
        ) : (
          <span className="flex items-center gap-2">
            <Loader2 size={16} className="animate-spin" /> {LABEL[order.status]}
            {order.estimated_delivery_at && (
              <span className="ml-2 text-xs opacity-75">
                ETA {new Date(order.estimated_delivery_at).toLocaleTimeString()}
              </span>
            )}
          </span>
        )}
      </div>

      {/* Live map */}
      {showMap && (
        <div className="mt-4 card overflow-hidden relative" style={{ height: 320 }}>
          <div className="absolute top-2 right-2 z-[500] flex items-center gap-1 rounded-full bg-white/90 px-2 py-1 text-xs font-medium shadow-sm">
            {isConnected ? (
              <span className="flex items-center gap-1 text-green-600">
                <Wifi size={12} /> Live
              </span>
            ) : (
              <span className="flex items-center gap-1 text-gray-500">
                <WifiOff size={12} /> Reconnecting…
              </span>
            )}
          </div>
          <LeafletMap
            pins={pins}
            driverPath={
              driverPath.length > 1
                ? driverPath.map((p) => ({ lat: p.lat, lng: p.lng }))
                : undefined
            }
            zoom={14}
          />
          {hasDriver && driverLocation && (
            <div className="absolute bottom-2 left-2 z-[500] rounded-lg bg-white/95 px-3 py-2 text-xs shadow-md flex items-center gap-2">
              <Bike size={14} className="text-blue-600" />
              <span>
                Driver en route
                {driverLocation.speed_kph != null && driverLocation.speed_kph > 0 && (
                  <span className="text-gray-500"> · {Math.round(driverLocation.speed_kph)} km/h</span>
                )}
              </span>
            </div>
          )}
        </div>
      )}

      {/* Timeline */}
      {!canceled && (
        <ol className="mt-6 space-y-3">
          {STEPS.map((step, i) => {
            const done = i < currentStepIdx;
            const active = i === currentStepIdx;
            return (
              <li key={step} className="flex items-center gap-3">
                <span
                  className={clsx(
                    "grid h-8 w-8 shrink-0 place-items-center rounded-full text-xs font-bold",
                    done
                      ? "bg-brand-500 text-white"
                      : active
                        ? "bg-brand-100 text-brand-700 ring-2 ring-brand-500"
                        : "bg-gray-100 text-gray-400",
                  )}
                >
                  {done ? <CheckCircle2 size={16} /> : i + 1}
                </span>
                <span
                  className={clsx(
                    "text-sm",
                    done
                      ? "text-gray-700"
                      : active
                        ? "font-semibold text-brand-700"
                        : "text-gray-400",
                  )}
                >
                  {LABEL[step]}
                </span>
              </li>
            );
          })}
        </ol>
      )}

      {/* Order items */}
      <section className="mt-8 card p-4">
        <h2 className="font-semibold mb-3">Items</h2>
        <ul className="space-y-2 text-sm">
          {order.items.map((item) => (
            <li key={item.id} className="flex justify-between">
              <span>
                <span className="text-gray-500">{item.quantity}×</span> {item.name}
              </span>
              <span>${(item.line_total_cents / 100).toFixed(2)}</span>
            </li>
          ))}
        </ul>
        <div className="mt-3 space-y-1 border-t border-gray-100 pt-3 text-sm">
          <div className="flex justify-between text-gray-600">
            <span>Subtotal</span>
            <span>${(order.subtotal_cents / 100).toFixed(2)}</span>
          </div>
          <div className="flex justify-between text-gray-600">
            <span>Delivery fee</span>
            <span>${(order.delivery_fee_cents / 100).toFixed(2)}</span>
          </div>
          {order.tip_cents > 0 && (
            <div className="flex justify-between text-gray-600">
              <span>Tip</span>
              <span>${(order.tip_cents / 100).toFixed(2)}</span>
            </div>
          )}
          <div className="flex justify-between font-semibold pt-1">
            <span>Total</span>
            <span>${(order.total_cents / 100).toFixed(2)}</span>
          </div>
        </div>
      </section>

      {/* Actions */}
      <div className="mt-4 flex gap-2">
        {!canceled && order.status !== "delivered" && (
          <Button
            variant="secondary"
            onClick={handleCancel}
            disabled={cancelOrder.isPending}
          >
            Cancel order
          </Button>
        )}
        <Button variant="ghost" as-child>
          <Link to="/orders">All orders</Link>
        </Button>
      </div>

      {lastEvent && (
        <p className="mt-4 text-xs text-gray-400">
          Last event: {lastEvent.event_type}
        </p>
      )}
    </div>
  );
}

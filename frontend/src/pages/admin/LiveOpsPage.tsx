import { useAnalytics, useLiveOrders, useLiveDrivers, useLiveRestaurants } from "@/hooks/useAdmin";
import { Package, Users, Store, Bike, DollarSign, Activity } from "lucide-react";
import { clsx } from "clsx";

export function LiveOpsPage() {
  const { data: analytics } = useAnalytics();
  const { data: orders } = useLiveOrders();
  const { data: drivers } = useLiveDrivers();
  const { data: restaurants } = useLiveRestaurants();

  return (
    <div className="mx-auto max-w-6xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Live Operations</h1>

      {/* KPI cards */}
      {analytics && (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          <KpiCard
            icon={Activity}
            label="Active orders"
            value={String(analytics.active_orders)}
            sub={`${analytics.total_orders} total`}
          />
          <KpiCard
            icon={Package}
            label="Delivered"
            value={String(analytics.delivered_orders)}
            sub={`${analytics.canceled_orders} canceled`}
          />
          <KpiCard
            icon={DollarSign}
            label="GMV"
            value={`$${(analytics.gmv_cents / 100).toFixed(0)}`}
            sub={`avg $${(analytics.avg_order_value_cents / 100).toFixed(2)}`}
          />
          <KpiCard
            icon={Users}
            label="Customers"
            value={String(analytics.total_customers)}
            sub={`${analytics.total_restaurants} restaurants · ${analytics.total_drivers} drivers`}
          />
        </div>
      )}

      {/* Active orders */}
      <section className="mt-8">
        <h2 className="font-display text-lg font-semibold mb-3">Active orders</h2>
        <div className="card overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-gray-50 text-xs text-gray-500">
              <tr>
                <th className="px-3 py-2 text-left">Order</th>
                <th className="px-3 py-2 text-left">Restaurant</th>
                <th className="px-3 py-2 text-left">Status</th>
                <th className="px-3 py-2 text-right">Total</th>
                <th className="px-3 py-2 text-left">Driver</th>
                <th className="px-3 py-2 text-left">Placed</th>
              </tr>
            </thead>
            <tbody>
              {orders?.map((o) => (
                <tr key={o.id} className="border-t border-gray-100">
                  <td className="px-3 py-2 font-mono text-xs">{o.id.slice(0, 8)}</td>
                  <td className="px-3 py-2">{o.restaurant_name}</td>
                  <td className="px-3 py-2">
                    <StatusBadge status={o.status} />
                  </td>
                  <td className="px-3 py-2 text-right">${(o.total_cents / 100).toFixed(2)}</td>
                  <td className="px-3 py-2">
                    {o.driver_id ? (
                      <span className="font-mono text-xs">{o.driver_id.slice(0, 8)}</span>
                    ) : (
                      <span className="text-gray-400">—</span>
                    )}
                  </td>
                  <td className="px-3 py-2 text-xs text-gray-500">
                    {new Date(o.placed_at).toLocaleTimeString()}
                  </td>
                </tr>
              ))}
              {orders?.length === 0 && (
                <tr>
                  <td colSpan={6} className="px-3 py-8 text-center text-gray-500">
                    No active orders right now.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>

      {/* Drivers + Restaurants grid */}
      <div className="mt-8 grid gap-6 lg:grid-cols-2">
        <section>
          <h2 className="font-display text-lg font-semibold mb-3 flex items-center gap-2">
            <Bike size={18} /> Online drivers ({drivers?.length ?? 0})
          </h2>
          <div className="space-y-1">
            {drivers?.map((d) => (
              <div key={d.id} className="card p-2 text-sm flex items-center justify-between">
                <div>
                  <span className="font-mono text-xs">{d.id.slice(0, 8)}</span>
                  <span className="ml-2 text-xs text-gray-500">{d.status}</span>
                </div>
                <div className="text-xs text-gray-500">
                  {d.total_deliveries} deliveries · {d.rating_avg?.toFixed(1) ?? "—"} ★
                </div>
              </div>
            ))}
            {drivers?.length === 0 && (
              <p className="text-sm text-gray-500 italic">No drivers online.</p>
            )}
          </div>
        </section>

        <section>
          <h2 className="font-display text-lg font-semibold mb-3 flex items-center gap-2">
            <Store size={18} /> Restaurants ({restaurants?.length ?? 0})
          </h2>
          <div className="space-y-1">
            {restaurants?.map((r) => (
              <div key={r.id} className="card p-2 text-sm flex items-center justify-between">
                <span>{r.name}</span>
                <StatusBadge status={r.status} />
              </div>
            ))}
            {restaurants?.length === 0 && (
              <p className="text-sm text-gray-500 italic">No restaurants yet.</p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

function KpiCard({
  icon: Icon,
  label,
  value,
  sub,
}: {
  icon: React.ComponentType<{ size?: number | string; className?: string }>;
  label: string;
  value: string;
  sub?: string;
}) {
  return (
    <div className="card p-4">
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-500">{label}</span>
        <Icon size={14} className="text-gray-400" />
      </div>
      <div className="mt-1 text-2xl font-semibold">{value}</div>
      {sub && <div className="text-xs text-gray-500">{sub}</div>}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    active: "bg-green-100 text-green-700",
    available: "bg-green-100 text-green-700",
    pending_accept: "bg-amber-100 text-amber-700",
    accepted: "bg-blue-100 text-blue-700",
    preparing: "bg-blue-100 text-blue-700",
    ready: "bg-indigo-100 text-indigo-700",
    picked_up: "bg-purple-100 text-purple-700",
    delivering: "bg-purple-100 text-purple-700",
    delivered: "bg-green-100 text-green-700",
    paused: "bg-amber-100 text-amber-700",
    closing: "bg-orange-100 text-orange-700",
    closed: "bg-red-100 text-red-700",
    assigned: "bg-blue-100 text-blue-700",
    offline: "bg-gray-100 text-gray-700",
  };
  return (
    <span
      className={clsx(
        "rounded-full px-2 py-0.5 text-xs font-medium capitalize",
        colors[status] ?? "bg-gray-100 text-gray-700",
      )}
    >
      {status.replace(/_/g, " ")}
    </span>
  );
}

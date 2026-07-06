import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/api/client";
import {
  Package,
  Users,
  Store,
  Bike,
  DollarSign,
  TrendingUp,
  Activity,
  XCircle,
} from "lucide-react";

interface Analytics {
  total_orders: number;
  active_orders: number;
  delivered_orders: number;
  canceled_orders: number;
  total_customers: number;
  total_restaurants: number;
  total_drivers: number;
  gmv_cents: number;
  avg_order_value_cents: number;
}

export function AnalyticsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ["admin", "analytics"],
    queryFn: async () => {
      const { data } = await apiClient.get<Analytics>("/admin/analytics/summary");
      return data;
    },
    refetchInterval: 60_000,
  });

  if (isLoading || !data) {
    return <div className="p-8 text-center text-gray-500">Loading…</div>;
  }

  const cancellationRate =
    data.total_orders > 0
      ? ((data.canceled_orders / data.total_orders) * 100).toFixed(1)
      : "0";

  return (
    <div className="mx-auto max-w-5xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Analytics</h1>

      {/* KPI grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4 mb-6">
        <Kpi
          icon={DollarSign}
          label="GMV"
          value={`$${(data.gmv_cents / 100).toFixed(0)}`}
          sub="gross merchandise value"
          color="text-green-600"
        />
        <Kpi
          icon={TrendingUp}
          label="Avg order value"
          value={`$${(data.avg_order_value_cents / 100).toFixed(2)}`}
          sub={`${data.delivered_orders} delivered`}
          color="text-blue-600"
        />
        <Kpi
          icon={Activity}
          label="Active orders"
          value={String(data.active_orders)}
          sub={`${data.total_orders} total`}
          color="text-amber-600"
        />
        <Kpi
          icon={XCircle}
          label="Cancellation rate"
          value={`${cancellationRate}%`}
          sub={`${data.canceled_orders} canceled`}
          color="text-red-600"
        />
      </div>

      {/* User counts */}
      <h2 className="font-display text-lg font-semibold mb-3">Platform users</h2>
      <div className="grid gap-4 sm:grid-cols-3 mb-6">
        <Kpi icon={Users} label="Customers" value={String(data.total_customers)} />
        <Kpi icon={Store} label="Restaurants" value={String(data.total_restaurants)} />
        <Kpi icon={Bike} label="Drivers" value={String(data.total_drivers)} />
      </div>

      {/* Order funnel */}
      <h2 className="font-display text-lg font-semibold mb-3">Order funnel</h2>
      <div className="card p-5">
        <div className="space-y-3">
          <FunnelBar
            label="Total placed"
            value={data.total_orders}
            max={data.total_orders}
            color="bg-gray-400"
          />
          <FunnelBar
            label="Delivered"
            value={data.delivered_orders}
            max={data.total_orders}
            color="bg-green-500"
          />
          <FunnelBar
            label="Active (in progress)"
            value={data.active_orders}
            max={data.total_orders}
            color="bg-blue-500"
          />
          <FunnelBar
            label="Canceled"
            value={data.canceled_orders}
            max={data.total_orders}
            color="bg-red-500"
          />
        </div>
      </div>

      {/* Charts placeholder */}
      <div className="mt-6 card p-8 text-center text-sm text-gray-500">
        <Package size={32} className="mx-auto mb-2 text-gray-300" />
        Time-series charts (GMV over time, daily orders, retention) require a
        time-bucketed analytics endpoint. Add `GET /admin/analytics/timeseries`
        with a Chart.js / Recharts frontend to visualize trends.
      </div>
    </div>
  );
}

function Kpi({
  icon: Icon,
  label,
  value,
  sub,
  color = "text-gray-700",
}: {
  icon: React.ComponentType<{ size?: number | string; className?: string }>;
  label: string;
  value: string;
  sub?: string;
  color?: string;
}) {
  return (
    <div className="card p-4">
      <div className="flex items-center justify-between">
        <span className="text-xs text-gray-500">{label}</span>
        <Icon size={14} className="text-gray-400" />
      </div>
      <div className={`mt-1 text-2xl font-bold ${color}`}>{value}</div>
      {sub && <div className="text-xs text-gray-500">{sub}</div>}
    </div>
  );
}

function FunnelBar({
  label,
  value,
  max,
  color,
}: {
  label: string;
  value: number;
  max: number;
  color: string;
}) {
  const pct = max > 0 ? (value / max) * 100 : 0;
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-gray-600">{label}</span>
        <span className="font-medium">
          {value} ({pct.toFixed(1)}%)
        </span>
      </div>
      <div className="h-2 rounded-full bg-gray-100 overflow-hidden">
        <div
          className={`h-full rounded-full ${color} transition-all`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}

import { Power, Bike, Star, Package, TrendingUp } from "lucide-react";
import { useDriverProfile, useGoOnline, useGoOffline } from "@/hooks/useDriver";
import { Button } from "@/components/ui/Button";
import { clsx } from "clsx";

export function ShiftPage() {
  const { data: driver, isLoading } = useDriverProfile();
  const goOnline = useGoOnline();
  const goOffline = useGoOffline();

  if (isLoading) return <div className="p-8 text-center text-gray-500">Loading…</div>;

  if (!driver) {
    return (
      <div className="mx-auto max-w-md px-4 py-12 text-center">
        <Bike size={48} className="mx-auto text-gray-300" />
        <h1 className="mt-3 font-display text-xl font-semibold">Become a driver</h1>
        <p className="mt-1 text-sm text-gray-500">
          Go online to start receiving delivery offers near you.
        </p>
        <Button className="mt-4" onClick={() => goOnline.mutate("bike")} disabled={goOnline.isPending}>
          <Power size={16} /> Go online
        </Button>
      </div>
    );
  }

  const isOnline = driver.status !== "offline";

  return (
    <div className="mx-auto max-w-md px-4 py-8">
      {/* Status banner */}
      <div
        className={clsx(
          "rounded-lg p-4 text-center font-semibold",
          isOnline ? "bg-green-50 text-green-700" : "bg-gray-100 text-gray-600",
        )}
      >
        <div className="flex items-center justify-center gap-2">
          <span
            className={clsx(
              "h-3 w-3 rounded-full",
              isOnline ? "bg-green-500 animate-pulse" : "bg-gray-400",
            )}
          />
          {isOnline ? "Online" : "Offline"}
        </div>
        <div className="mt-1 text-xs text-gray-500 capitalize">
          {driver.status.replace("_", " ")}
        </div>
      </div>

      {/* Stats */}
      <div className="mt-6 grid grid-cols-3 gap-3">
        <StatCard icon={Star} label="Rating" value={driver.rating_avg?.toFixed(1) ?? "—"} />
        <StatCard icon={Package} label="Deliveries" value={String(driver.total_deliveries)} />
        <StatCard
          icon={TrendingUp}
          label="Acceptance"
          value={`${Math.round(driver.acceptance_rate * 100)}%`}
        />
      </div>

      {/* Active order */}
      {driver.current_order_id && (
        <div className="mt-6 card p-4 border-l-4 border-brand-500">
          <div className="text-sm text-gray-500">Active order</div>
          <div className="mt-1 font-semibold">#{driver.current_order_id.slice(0, 8)}</div>
          <Button className="mt-3 w-full" as-child>
            <a href={`/driver/active`}>View delivery</a>
          </Button>
        </div>
      )}

      {/* Toggle button */}
      <Button
        className="mt-6 w-full"
        size="lg"
        variant={isOnline ? "secondary" : "primary"}
        onClick={() => (isOnline ? goOffline.mutate() : goOnline.mutate(undefined))}
        disabled={goOnline.isPending || goOffline.isPending}
      >
        <Power size={16} />
        {isOnline ? "Go offline" : "Go online"}
      </Button>

      {!driver.current_order_id && isOnline && (
        <p className="mt-3 text-center text-xs text-gray-500">
          You'll receive order offers here when restaurants mark food ready.
        </p>
      )}

      {/* Vehicle info */}
      <div className="mt-6 card p-4 text-sm">
        <div className="flex justify-between">
          <span className="text-gray-500">Vehicle</span>
          <span className="capitalize">{driver.vehicle_type}</span>
        </div>
        {driver.license_plate && (
          <div className="flex justify-between mt-1">
            <span className="text-gray-500">Plate</span>
            <span>{driver.license_plate}</span>
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
}: {
  icon: React.ComponentType<{ size?: number | string; className?: string }>;
  label: string;
  value: string;
}) {
  return (
    <div className="card p-3 text-center">
      <Icon size={16} className="mx-auto text-gray-400" />
      <div className="mt-1 text-lg font-semibold">{value}</div>
      <div className="text-xs text-gray-500">{label}</div>
    </div>
  );
}

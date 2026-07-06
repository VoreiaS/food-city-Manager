import { useState } from "react";
import { Link } from "react-router-dom";
import { UtensilsCrossed, ClipboardList, Star, Wallet, Settings, Power } from "lucide-react";
import { useRestaurantProfile, useUpdateRestaurantStatus } from "@/hooks/useRestaurant";
import { Button } from "@/components/ui/Button";
import { clsx } from "clsx";
import type { RestaurantStatus } from "@/api/restaurant.api";

export function DashboardPage() {
  const { data: profile, isLoading } = useRestaurantProfile();
  const updateStatus = useUpdateRestaurantStatus();
  const [showStatusMenu, setShowStatusMenu] = useState(false);

  if (isLoading || !profile) {
    return <div className="p-8 text-center text-gray-500">Loading…</div>;
  }

  const statusOptions: { value: RestaurantStatus; label: string; color: string }[] = [
    { value: "active", label: "Accepting orders", color: "bg-green-500" },
    { value: "paused", label: "Paused for today", color: "bg-amber-500" },
    { value: "closing", label: "Closing (finish in-flight)", color: "bg-orange-500" },
    { value: "closed", label: "Closed", color: "bg-red-500" },
  ];

  const currentStatus = statusOptions.find((s) => s.value === profile.status);

  return (
    <div className="mx-auto max-w-6xl px-4 py-8">
      <div className="flex items-start justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold">{profile.name}</h1>
          <div className="mt-1 flex items-center gap-3 text-sm text-gray-500">
            <span className="flex items-center gap-1">
              <Star size={14} fill="currentColor" className="text-amber-500" />
              {profile.rating_avg?.toFixed(1) ?? "—"} ({profile.rating_count})
            </span>
            <span>{profile.cuisine_types.join(", ")}</span>
          </div>
        </div>

        {/* Status toggle */}
        <div className="relative">
          <Button
            variant="secondary"
            onClick={() => setShowStatusMenu((s) => !s)}
            className="flex items-center gap-2"
          >
            <span className={clsx("h-2 w-2 rounded-full", currentStatus?.color)} />
            {currentStatus?.label ?? profile.status}
            <Power size={14} />
          </Button>
          {showStatusMenu && (
            <div className="absolute right-0 top-full z-10 mt-1 w-56 rounded-lg border border-gray-200 bg-white p-1 shadow-md">
              {statusOptions.map((s) => (
                <button
                  key={s.value}
                  onClick={() => {
                    updateStatus.mutate(s.value);
                    setShowStatusMenu(false);
                  }}
                  className={clsx(
                    "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-gray-50",
                    profile.status === s.value && "bg-gray-50",
                  )}
                >
                  <span className={clsx("h-2 w-2 rounded-full", s.color)} />
                  {s.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Stats grid */}
      <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {[
          { label: "Today's orders", value: "—", icon: ClipboardList },
          { label: "Pending accept", value: "—", icon: UtensilsCrossed },
          { label: "Avg rating", value: profile.rating_avg?.toFixed(1) ?? "—", icon: Star },
          { label: "This week's earnings", value: "—", icon: Wallet },
        ].map((stat) => (
          <div key={stat.label} className="card p-4">
            <div className="flex items-center justify-between">
              <span className="text-xs text-gray-500">{stat.label}</span>
              <stat.icon size={16} className="text-gray-400" />
            </div>
            <div className="mt-2 text-2xl font-semibold">{stat.value}</div>
          </div>
        ))}
      </div>

      {/* Quick links */}
      <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Link
          to="/restaurant/orders"
          className="card flex items-center gap-3 p-4 hover:shadow-md"
        >
          <ClipboardList size={20} className="text-brand-500" />
          <div>
            <div className="font-semibold">Orders</div>
            <div className="text-xs text-gray-500">Manage incoming orders</div>
          </div>
        </Link>
        <Link to="/restaurant/menu" className="card flex items-center gap-3 p-4 hover:shadow-md">
          <UtensilsCrossed size={20} className="text-brand-500" />
          <div>
            <div className="font-semibold">Menu</div>
            <div className="text-xs text-gray-500">Manage items & categories</div>
          </div>
        </Link>
        <Link to="/restaurant/reviews" className="card flex items-center gap-3 p-4 hover:shadow-md">
          <Star size={20} className="text-brand-500" />
          <div>
            <div className="font-semibold">Reviews</div>
            <div className="text-xs text-gray-500">View & reply to reviews</div>
          </div>
        </Link>
        <Link to="/restaurant/earnings" className="card flex items-center gap-3 p-4 hover:shadow-md">
          <Wallet size={20} className="text-brand-500" />
          <div>
            <div className="font-semibold">Earnings</div>
            <div className="text-xs text-gray-500">Payouts & statements</div>
          </div>
        </Link>
      </div>

      <div className="mt-8 card p-4 flex items-center gap-2 text-sm text-gray-500">
        <Settings size={14} />
        Restaurant settings (hours, delivery radius, fees) editable via the Profile API.
      </div>
    </div>
  );
}

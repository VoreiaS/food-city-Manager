import { clsx } from "clsx";
import { Star, Clock, Bike } from "lucide-react";
import { Link } from "react-router-dom";
import type { RestaurantCard as RestaurantCardType } from "@/api/restaurants.api";
import { SafeImage } from "@/components/common/SafeImage";

interface Props {
  restaurant: RestaurantCardType;
}

export function RestaurantCardItem({ restaurant: r }: Props) {
  return (
    <Link
      to={`/restaurants/${r.id}`}
      className="card block overflow-hidden transition hover:shadow-md"
    >
      <div className="relative h-32 bg-gradient-to-br from-brand-100 to-amber-100">
        <SafeImage
          src={r.cover_url}
          alt={r.name}
          fallback="store"
          className="h-full w-full object-cover"
        />
        <span
          className={clsx(
            "absolute top-2 right-2 rounded-full px-2 py-0.5 text-xs font-medium",
            r.is_open
              ? "bg-green-500 text-white"
              : "bg-gray-700 text-white",
          )}
        >
          {r.is_open ? "Open" : "Closed"}
        </span>
      </div>
      <div className="p-3">
        <div className="flex items-start justify-between gap-2">
          <h3 className="font-display font-semibold leading-tight">{r.name}</h3>
          {r.rating_avg != null && (
            <span className="flex shrink-0 items-center gap-1 text-xs font-medium text-amber-700">
              <Star size={12} fill="currentColor" />
              {r.rating_avg.toFixed(1)}
            </span>
          )}
        </div>
        <p className="mt-1 line-clamp-2 text-xs text-gray-500">
          {r.description ?? r.cuisine_types.join(", ")}
        </p>
        <div className="mt-2 flex items-center gap-3 text-xs text-gray-500">
          <span className="flex items-center gap-1">
            <Bike size={12} />
            ${(r.delivery_fee_cents / 100).toFixed(2)}
          </span>
          {r.delivery_eta_min != null && (
            <span className="flex items-center gap-1">
              <Clock size={12} />
              {r.delivery_eta_min} min
            </span>
          )}
          {r.distance_m != null && (
            <span>{(r.distance_m / 1000).toFixed(1)} km</span>
          )}
        </div>
      </div>
    </Link>
  );
}

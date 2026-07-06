import { useEffect, useRef } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

// Fix default marker icons (Leaflet's CSS references images that Vite can't resolve)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
delete (L.Icon.Default.prototype as any)._getIconUrl;
L.Icon.Default.mergeOptions({
  iconRetinaUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png",
  iconUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png",
  shadowUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png",
});

export interface MapPin {
  lat: number;
  lng: number;
  label?: string;
  type: "restaurant" | "customer" | "driver";
}

interface Props {
  pins: MapPin[];
  center?: { lat: number; lng: number };
  zoom?: number;
  className?: string;
  /** Optional: animate driver pin between positions */
  driverPath?: { lat: number; lng: number }[];
}

const pinColors: Record<MapPin["type"], string> = {
  restaurant: "#f97316", // brand orange
  customer: "#10b981", // green
  driver: "#3b82f6", // blue
};

const pinIcons: Record<MapPin["type"], string> = {
  restaurant: "🍽️",
  customer: "🏠",
  driver: "🛵",
};

function createDivIcon(type: MapPin["type"], label?: string): L.DivIcon {
  const color = pinColors[type];
  const emoji = pinIcons[type];
  const html = `
    <div class="relative flex flex-col items-center">
      <div style="background:${color};width:32px;height:32px;border-radius:50% 50% 50% 0;transform:rotate(-45deg);display:flex;align-items:center;justify-content:center;box-shadow:0 2px 6px rgba(0,0,0,0.3);border:2px solid white;">
        <span style="transform:rotate(45deg);font-size:14px;">${emoji}</span>
      </div>
      ${label ? `<div style="background:white;padding:2px 6px;border-radius:4px;font-size:11px;font-weight:600;margin-top:2px;box-shadow:0 1px 3px rgba(0,0,0,0.2);white-space:nowrap;">${label}</div>` : ""}
    </div>
  `;
  return L.divIcon({
    html,
    className: "food-city-map-pin",
    iconSize: [32, 32],
    iconAnchor: [16, 32],
    popupAnchor: [0, -32],
  });
}

export function LeafletMap({ pins, center, zoom = 13, className, driverPath }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const mapRef = useRef<L.Map | null>(null);
  const markersRef = useRef<Map<string, L.Marker>>(new Map());
  const pathRef = useRef<L.Polyline | null>(null);

  // Initialize map once
  useEffect(() => {
    if (!containerRef.current || mapRef.current) return;
    const map = L.map(containerRef.current, {
      center: [center?.lat ?? 6.9271, center?.lng ?? 79.8612],
      zoom,
      zoomControl: true,
      attributionControl: true,
    });
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
      maxZoom: 19,
    }).addTo(map);
    mapRef.current = map;
    // Copy refs to local vars for cleanup (avoids stale-ref warning)
    const markers = markersRef.current;
    return () => {
      const m = mapRef.current;
      if (m) {
        m.remove();
      }
      mapRef.current = null;
      markers.clear();
      pathRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Update markers when pins change
  useEffect(() => {
    const map = mapRef.current;
    if (!map) return;

    // Remove markers that are no longer present
    for (const [key, marker] of markersRef.current.entries()) {
      if (!pins.find((p) => pinKey(p) === key)) {
        map.removeLayer(marker);
        markersRef.current.delete(key);
      }
    }

    // Add or update markers
    for (const pin of pins) {
      const key = pinKey(pin);
      const existing = markersRef.current.get(key);
      if (existing) {
        // Update position (animates via setLatLng)
        existing.setLatLng([pin.lat, pin.lng]);
      } else {
        const marker = L.marker([pin.lat, pin.lng], {
          icon: createDivIcon(pin.type, pin.label),
        }).addTo(map);
        markersRef.current.set(key, marker);
      }
    }

    // Fit bounds to show all pins
    if (pins.length > 1) {
      const bounds = L.latLngBounds(pins.map((p) => [p.lat, p.lng] as [number, number]));
      map.fitBounds(bounds, { padding: [50, 50], maxZoom: 15 });
    } else if (pins.length === 1 && pins[0]) {
      map.setView([pins[0].lat, pins[0].lng], zoom);
    }
  }, [pins, zoom]);

  // Draw driver path if provided
  useEffect(() => {
    const map = mapRef.current;
    if (!map) return;
    if (pathRef.current) {
      map.removeLayer(pathRef.current);
      pathRef.current = null;
    }
    if (driverPath && driverPath.length > 1) {
      pathRef.current = L.polyline(
        driverPath.map((p) => [p.lat, p.lng] as [number, number]),
        { color: "#3b82f6", weight: 3, opacity: 0.6, dashArray: "8 6" },
      ).addTo(map);
    }
  }, [driverPath]);

  return <div ref={containerRef} className={className} style={{ height: "100%", width: "100%" }} />;
}

function pinKey(pin: MapPin): string {
  return `${pin.type}:${pin.label ?? ""}`;
}

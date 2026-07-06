import { useEffect, useState, useCallback } from "react";
import { useAuthStore } from "@/store/authStore";
import { getWsClient, type WsClient } from "@/utils/ws";

// Re-export for convenience
export { getWsClient };

interface OrderEvent {
  type: string;
  channel: string;
  event_type?: string;
  payload?: {
    sequence?: number;
    order_id?: string;
    status?: string;
    [k: string]: unknown;
  };
  [k: string]: unknown;
}

interface DriverLocationEvent {
  type: "driver_location";
  channel: string;
  driver_id: string;
  lat: number;
  lng: number;
  heading?: number;
  speed_kph?: number;
}

export interface DriverLocation {
  lat: number;
  lng: number;
  heading?: number;
  speed_kph?: number;
  timestamp: number;
}

/**
 * Subscribe to a single order's realtime events. Returns the latest event
 * received (or null). Auto-reconnects on WS drop, replays missed events.
 */
export function useOrderTracking(orderId: string | undefined) {
  const accessToken = useAuthStore((s) => s.accessToken);
  const [lastEvent, setLastEvent] = useState<OrderEvent | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    if (!orderId || !accessToken) return;
    const client = getWsClient(accessToken);
    const channel = `order:${orderId}:events`;
    const handler = (msg: Record<string, unknown>) => {
      setLastEvent(msg as unknown as OrderEvent);
      setIsConnected(true);
    };
    client.subscribe(channel, handler);
    return () => {
      client.unsubscribe(channel, handler);
    };
  }, [orderId, accessToken]);

  return { lastEvent, isConnected };
}

/**
 * Subscribe to driver location updates for an order.
 * Returns the latest driver location (or null if no driver assigned yet).
 */
export function useDriverLocation(orderId: string | undefined) {
  const accessToken = useAuthStore((s) => s.accessToken);
  const [location, setLocation] = useState<DriverLocation | null>(null);
  const [path, setPath] = useState<DriverLocation[]>([]);

  useEffect(() => {
    if (!orderId || !accessToken) return;
    const client = getWsClient(accessToken);
    const channel = `order:${orderId}:events`;
    const handler = (msg: Record<string, unknown>) => {
      if (msg.type === "driver_location" && "lat" in msg && "lng" in msg) {
        const locMsg = msg as unknown as DriverLocationEvent;
        const loc: DriverLocation = {
          lat: locMsg.lat,
          lng: locMsg.lng,
          heading: locMsg.heading,
          speed_kph: locMsg.speed_kph,
          timestamp: Date.now(),
        };
        setLocation(loc);
        setPath((prev) => {
          const next = [...prev, loc];
          // Keep last 50 points to avoid unbounded growth
          return next.slice(-50);
        });
      }
    };
    client.subscribe(channel, handler);
    return () => {
      client.unsubscribe(channel, handler);
    };
  }, [orderId, accessToken]);

  const resetPath = useCallback(() => setPath([]), []);

  return { location, path, resetPath };
}

export type { OrderEvent, DriverLocationEvent, WsClient };

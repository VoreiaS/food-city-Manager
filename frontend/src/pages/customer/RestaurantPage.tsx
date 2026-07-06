import { useParams } from "react-router-dom";
import { RestaurantDetail } from "@/components/restaurant/RestaurantDetail";

export function RestaurantPage() {
  const { id } = useParams<{ id: string }>();
  if (!id) return <div className="p-8 text-center text-gray-500">No restaurant ID.</div>;
  return <RestaurantDetail restaurantId={id} />;
}

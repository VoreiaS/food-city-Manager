import { Link } from "react-router-dom";
import { Home, UtensilsCrossed } from "lucide-react";
import { Button } from "@/components/ui/Button";

export function NotFoundPage() {
  return (
    <div className="min-h-[60vh] flex items-center justify-center px-4">
      <div className="text-center">
        <div className="text-8xl font-display font-bold text-brand-200">404</div>
        <h1 className="mt-2 font-display text-2xl font-bold">Page not found</h1>
        <p className="mt-2 text-sm text-gray-500">
          The page you're looking for doesn't exist or has been moved.
        </p>
        <div className="mt-6 flex gap-2 justify-center">
          <Button as-child>
            <Link to="/">
              <Home size={14} /> Go home
            </Link>
          </Button>
          <Button variant="secondary" as-child>
            <Link to="/restaurants">
              <UtensilsCrossed size={14} /> Browse restaurants
            </Link>
          </Button>
        </div>
      </div>
    </div>
  );
}

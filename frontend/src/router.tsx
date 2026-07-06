import { createBrowserRouter, Navigate } from "react-router-dom";
import { LayoutPage } from "@/components/layout/LayoutPage";
import { HomePage } from "@/pages/customer/HomePage";
import { LoginPage } from "@/pages/auth/LoginPage";
import { RegisterPage } from "@/pages/auth/RegisterPage";
import { RestaurantPage } from "@/pages/customer/RestaurantPage";
import { CheckoutPage } from "@/pages/customer/CheckoutPage";
import { OrderTrackingPage } from "@/pages/customer/OrderTrackingPage";
import { OrdersPage } from "@/pages/customer/OrdersPage";
import { ProfilePage } from "@/pages/customer/ProfilePage";
import { LoyaltyPage } from "@/pages/customer/LoyaltyPage";
import { DashboardPage as RestaurantDashboard } from "@/pages/restaurant/DashboardPage";
import { OrdersPage as RestaurantOrders } from "@/pages/restaurant/OrdersPage";
import { MenuPage } from "@/pages/restaurant/MenuPage";
import { ReviewsPage } from "@/pages/restaurant/ReviewsPage";
import { EarningsPage as RestaurantEarnings } from "@/pages/restaurant/EarningsPage";
import { ShiftPage } from "@/pages/driver/ShiftPage";
import { OrderOfferPage } from "@/pages/driver/OrderOfferPage";
import { ActiveDeliveryPage } from "@/pages/driver/ActiveDeliveryPage";
import { EarningsPage as DriverEarnings } from "@/pages/driver/EarningsPage";
import { LiveOpsPage } from "@/pages/admin/LiveOpsPage";
import { VerificationsPage } from "@/pages/admin/VerificationsPage";
import { DisputesPage } from "@/pages/admin/DisputesPage";
import { AnalyticsPage } from "@/pages/admin/AnalyticsPage";
import { useAuthStore } from "@/store/authStore";
import type { UserRole } from "@/types";
import { NotFoundPage } from "@/pages/common/NotFoundPage";

// eslint-disable-next-line react-refresh/only-export-components
function RequireRole({ role, children }: { role: UserRole | UserRole[]; children: React.ReactNode }) {
  const user = useAuthStore((s) => s.user);
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  if (!isAuthenticated || !user) return <Navigate to="/login" replace />;
  const roles = Array.isArray(role) ? role : [role];
  if (!roles.includes(user.role)) return <Navigate to="/" replace />;
  return <>{children}</>;
}

export const router = createBrowserRouter([
  {
    path: "/",
    element: <LayoutPage />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "login", element: <LoginPage /> },
      { path: "register", element: <RegisterPage /> },

      // Customer-only
      { path: "restaurants", element: <RestaurantPage /> },
      { path: "checkout", element: <RequireRole role="customer"><CheckoutPage /></RequireRole> },
      { path: "orders/:id/track", element: <RequireRole role="customer"><OrderTrackingPage /></RequireRole> },
      { path: "orders", element: <RequireRole role="customer"><OrdersPage /></RequireRole> },
      { path: "profile", element: <RequireRole role="customer"><ProfilePage /></RequireRole> },
      { path: "loyalty", element: <RequireRole role="customer"><LoyaltyPage /></RequireRole> },

      // Restaurant-only
      { path: "restaurant", element: <RequireRole role="restaurant"><RestaurantDashboard /></RequireRole> },
      { path: "restaurant/orders", element: <RequireRole role="restaurant"><RestaurantOrders /></RequireRole> },
      { path: "restaurant/menu", element: <RequireRole role="restaurant"><MenuPage /></RequireRole> },
      { path: "restaurant/reviews", element: <RequireRole role="restaurant"><ReviewsPage /></RequireRole> },
      { path: "restaurant/earnings", element: <RequireRole role="restaurant"><RestaurantEarnings /></RequireRole> },

      // Driver-only
      { path: "driver", element: <RequireRole role="driver"><ShiftPage /></RequireRole> },
      { path: "driver/offer/:orderId", element: <RequireRole role="driver"><OrderOfferPage /></RequireRole> },
      { path: "driver/active", element: <RequireRole role="driver"><ActiveDeliveryPage /></RequireRole> },
      { path: "driver/earnings", element: <RequireRole role="driver"><DriverEarnings /></RequireRole> },

      // Admin-only
      { path: "admin", element: <RequireRole role="admin"><LiveOpsPage /></RequireRole> },
      { path: "admin/verifications", element: <RequireRole role="admin"><VerificationsPage /></RequireRole> },
      { path: "admin/disputes", element: <RequireRole role="admin"><DisputesPage /></RequireRole> },
      { path: "admin/analytics", element: <RequireRole role="admin"><AnalyticsPage /></RequireRole> },

      // 404 catch-all
      { path: "*", element: <NotFoundPage /> },
    ],
  },
]);

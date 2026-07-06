import { Outlet } from "react-router-dom";
import { Layout } from "@/components/layout/Header";
import { CartDrawer } from "@/components/cart/CartDrawer";

export function LayoutPage() {
  return (
    <Layout>
      <Outlet />
      <CartDrawer />
    </Layout>
  );
}

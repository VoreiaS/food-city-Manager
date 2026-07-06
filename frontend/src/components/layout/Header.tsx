import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { LogOut, User, UtensilsCrossed, Truck, ShieldCheck, ShoppingBag, Menu, X } from "lucide-react";
import { useAuthStore } from "@/store/authStore";
import { useCartStore } from "@/store/cartStore";
import { Button } from "@/components/ui/Button";

export function Header() {
  const { user, isAuthenticated, logout } = useAuthStore();
  const navigate = useNavigate();
  const cart = useCartStore((s) => s.cart);
  const openDrawer = useCartStore((s) => s.openDrawer);
  const [mobileOpen, setMobileOpen] = useState(false);

  const handleLogout = () => {
    logout();
    navigate("/");
    setMobileOpen(false);
  };

  const navLinks = (
    <>
      <Button variant="ghost" size="sm" as-child>
        <Link to="/" onClick={() => setMobileOpen(false)}>Browse</Link>
      </Button>
      {isAuthenticated && user?.role === "customer" && (
        <Button variant="ghost" size="sm" as-child>
          <Link to="/orders" onClick={() => setMobileOpen(false)}>Orders</Link>
        </Button>
      )}
      {isAuthenticated && user?.role === "customer" && (
        <Button variant="ghost" size="sm" as-child>
          <Link to="/loyalty" onClick={() => setMobileOpen(false)}>Loyalty</Link>
        </Button>
      )}
      {isAuthenticated && user?.role === "restaurant" && (
        <Button variant="ghost" size="sm" as-child>
          <Link to="/restaurant" onClick={() => setMobileOpen(false)}>Dashboard</Link>
        </Button>
      )}
      {isAuthenticated && user?.role === "driver" && (
        <Button variant="ghost" size="sm" as-child>
          <Link to="/driver" onClick={() => setMobileOpen(false)}>Shift</Link>
        </Button>
      )}
      {isAuthenticated && user?.role === "admin" && (
        <Button variant="ghost" size="sm" as-child>
          <Link to="/admin" onClick={() => setMobileOpen(false)}>Admin</Link>
        </Button>
      )}
    </>
  );

  return (
    <header className="sticky top-0 z-30 border-b border-gray-100 bg-white/90 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4">
        <Link to="/" className="flex items-center gap-2 font-display text-lg font-bold">
          <span className="grid h-9 w-9 place-items-center rounded-lg bg-brand-500 text-white">
            <UtensilsCrossed size={20} />
          </span>
          <span>Food City</span>
        </Link>

        {/* Desktop nav */}
        <nav className="hidden items-center gap-2 md:flex">
          {navLinks}
        </nav>

        <div className="flex items-center gap-2">
          {isAuthenticated && user?.role === "customer" && cart && cart.items.length > 0 && (
            <Button variant="secondary" size="sm" onClick={openDrawer} className="relative">
              <ShoppingBag size={14} />
              <span className="hidden sm:inline">Cart</span>
              <span className="absolute -top-1 -right-1 grid h-5 min-w-5 place-items-center rounded-full bg-brand-500 px-1 text-xs text-white">
                {cart.items.length}
              </span>
            </Button>
          )}
          {isAuthenticated && user ? (
            <>
              <div className="hidden items-center gap-2 sm:flex">
                <span className="grid h-8 w-8 place-items-center rounded-full bg-gray-100">
                  <User size={16} />
                </span>
                <div className="text-sm">
                  <div className="font-medium">{user.full_name}</div>
                  <div className="text-xs text-gray-500 capitalize">{user.role}</div>
                </div>
              </div>
              <Button variant="ghost" size="sm" onClick={handleLogout} aria-label="Log out">
                <LogOut size={16} />
              </Button>
            </>
          ) : (
            <>
              <Button variant="ghost" size="sm" onClick={() => navigate("/login")}>
                Log in
              </Button>
              <Button size="sm" onClick={() => navigate("/register")}>
                Sign up
              </Button>
            </>
          )}
          {/* Mobile menu toggle */}
          <button
            className="md:hidden rounded p-2 hover:bg-gray-100"
            onClick={() => setMobileOpen((s) => !s)}
            aria-label="Toggle menu"
          >
            {mobileOpen ? <X size={18} /> : <Menu size={18} />}
          </button>
        </div>
      </div>

      {/* Mobile nav drawer */}
      {mobileOpen && (
        <div className="md:hidden border-t border-gray-100 bg-white px-4 py-3 space-y-1">
          {navLinks}
          {isAuthenticated && (
            <Button variant="ghost" size="sm" as-child>
              <Link to="/profile" onClick={() => setMobileOpen(false)}>Profile</Link>
            </Button>
          )}
        </div>
      )}
    </header>
  );
}

export function Footer() {
  return (
    <footer className="border-t border-gray-100 bg-white">
      <div className="mx-auto max-w-7xl px-4 py-8 text-sm text-gray-500">
        <div className="flex flex-wrap gap-8">
          <div>
            <h4 className="mb-2 font-semibold text-gray-700">Food City</h4>
            <p>Discover · Order · Track</p>
          </div>
          <div>
            <h4 className="mb-2 font-semibold text-gray-700">For Customers</h4>
            <ul className="space-y-1">
              <li>Browse restaurants</li>
              <li>Track orders</li>
              <li>Loyalty rewards</li>
            </ul>
          </div>
          <div>
            <h4 className="mb-2 font-semibold text-gray-700">For Partners</h4>
            <ul className="space-y-1">
              <li className="flex items-center gap-1"><UtensilsCrossed size={12} /> Restaurant</li>
              <li className="flex items-center gap-1"><Truck size={12} /> Driver</li>
              <li className="flex items-center gap-1"><ShieldCheck size={12} /> Admin</li>
            </ul>
          </div>
        </div>
        <div className="mt-6 border-t border-gray-100 pt-4 text-xs">
          © {new Date().getFullYear()} Food City. Built with Rust + React.
        </div>
      </div>
    </footer>
  );
}

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="flex-1">{children}</main>
      <Footer />
    </div>
  );
}

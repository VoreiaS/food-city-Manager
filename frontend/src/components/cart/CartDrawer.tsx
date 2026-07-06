import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { X, ShoppingBag, Plus, Minus, Trash2 } from "lucide-react";
import { useCartStore } from "@/store/cartStore";
import { useAuthStore } from "@/store/authStore";
import { Button } from "@/components/ui/Button";

export function CartDrawer() {
  const navigate = useNavigate();
  const { cart, isDrawerOpen, closeDrawer, updateItem, removeItem, clearCart } = useCartStore();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);

  // Lock body scroll when open
  useEffect(() => {
    if (isDrawerOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [isDrawerOpen]);

  if (!isDrawerOpen) return null;

  const handleCheckout = () => {
    closeDrawer();
    if (!isAuthenticated) {
      navigate("/login?redirect=/checkout");
    } else {
      navigate("/checkout");
    }
  };

  return (
    <div className="fixed inset-0 z-40 flex justify-end bg-black/40" onClick={closeDrawer}>
      <div
        className="flex h-full w-full max-w-md flex-col bg-white"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-gray-100 px-4 py-3">
          <h2 className="flex items-center gap-2 font-display font-semibold">
            <ShoppingBag size={18} /> Your Cart
          </h2>
          <button onClick={closeDrawer} className="rounded p-1 hover:bg-gray-100">
            <X size={18} />
          </button>
        </div>

        {!cart || cart.items.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center text-gray-500">
            <ShoppingBag size={48} className="text-gray-300" />
            <p>Your cart is empty.</p>
            <Button variant="secondary" onClick={closeDrawer}>
              Browse restaurants
            </Button>
          </div>
        ) : (
          <>
            <div className="flex items-center justify-between bg-gray-50 px-4 py-2 text-sm">
              <span className="font-medium">{cart.restaurant_name}</span>
              <button
                onClick={() => {
                  if (confirm("Clear cart?")) clearCart();
                }}
                className="text-xs text-red-500 hover:underline"
              >
                Clear
              </button>
            </div>

            <div className="flex-1 overflow-y-auto px-4 py-3">
              {cart.items.map((item) => (
                <div key={item.id} className="border-b border-gray-100 py-3 last:border-0">
                  <div className="flex gap-3">
                    <div className="flex-1">
                      <h3 className="text-sm font-medium">{item.menu_item_name}</h3>
                      {item.customizations.length > 0 && (
                        <ul className="mt-0.5 text-xs text-gray-500">
                          {item.customizations.map((c) => (
                            <li key={c.option_id}>
                              {c.customization_name}: {c.option_name}
                              {c.price_cents > 0 &&
                                ` (+$${(c.price_cents / 100).toFixed(2)})`}
                            </li>
                          ))}
                        </ul>
                      )}
                      {item.notes && (
                        <p className="mt-0.5 text-xs italic text-gray-500">"{item.notes}"</p>
                      )}
                      <p className="mt-1 text-sm font-medium text-brand-700">
                        ${(item.line_total_cents / 100).toFixed(2)}
                      </p>
                    </div>
                    <div className="flex flex-col items-end gap-2">
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() =>
                            updateItem(item.id, {
                              quantity: Math.max(1, item.quantity - 1),
                            })
                          }
                          className="grid h-7 w-7 place-items-center rounded border border-gray-200 hover:bg-gray-50"
                        >
                          <Minus size={12} />
                        </button>
                        <span className="w-6 text-center text-sm">{item.quantity}</span>
                        <button
                          onClick={() =>
                            updateItem(item.id, { quantity: item.quantity + 1 })
                          }
                          className="grid h-7 w-7 place-items-center rounded border border-gray-200 hover:bg-gray-50"
                        >
                          <Plus size={12} />
                        </button>
                      </div>
                      <button
                        onClick={() => removeItem(item.id)}
                        className="text-xs text-red-500 hover:underline"
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            <div className="border-t border-gray-100 px-4 py-3 space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">Subtotal</span>
                <span>${(cart.subtotal_cents / 100).toFixed(2)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-600">Delivery fee</span>
                <span>${(cart.delivery_fee_cents / 100).toFixed(2)}</span>
              </div>
              <div className="flex justify-between font-semibold">
                <span>Total</span>
                <span>${(cart.total_cents / 100).toFixed(2)}</span>
              </div>
              {!cart.meets_min_order && (
                <p className="text-xs text-amber-700">
                  Minimum order is ${(cart.min_order_cents / 100).toFixed(2)}.
                  Add ${(cart.min_order_cents - cart.subtotal_cents / 1).toFixed(2)} more.
                </p>
              )}
              <Button
                onClick={handleCheckout}
                disabled={!cart.meets_min_order}
                className="w-full"
                size="lg"
              >
                Go to Checkout
              </Button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { CreditCard, MapPin, Plus, Loader2, Tag, CheckCircle2 } from "lucide-react";
import { useCartStore } from "@/store/cartStore";
import { useAddresses, useCreateAddress } from "@/hooks/useAddresses";
import { usePlaceOrder } from "@/hooks/useOrders";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { clsx } from "clsx";

export function CheckoutPage() {
  const navigate = useNavigate();
  const cart = useCartStore((s) => s.cart);
  const clearCart = useCartStore((s) => s.clearCart);
  const { data: addresses, isLoading: addrLoading } = useAddresses();
  const createAddr = useCreateAddress();
  const placeOrder = usePlaceOrder();

  const [selectedAddrId, setSelectedAddrId] = useState<string>("");
  const [tipCents, setTipCents] = useState(0);
  const [notes, setNotes] = useState("");
  const [showAddrForm, setShowAddrForm] = useState(false);
  const [paymentMode, setPaymentMode] = useState<"mock" | "stripe">("mock");
  const [newAddr, setNewAddr] = useState({
    label: "Home",
    line1: "",
    city: "Colombo",
    lat: 6.9271,
    lng: 79.8612,
    formatted_address: "",
  });
  const [submitting, setSubmitting] = useState(false);

  if (!cart || cart.items.length === 0) {
    return (
      <div className="mx-auto max-w-md px-4 py-12 text-center">
        <p className="text-gray-500">Your cart is empty.</p>
        <Button className="mt-4" onClick={() => navigate("/")}>
          Browse restaurants
        </Button>
      </div>
    );
  }

  // Check if restaurant might be closed (we don't have is_open in cart response,
  // but if status is abandoned/locked, block checkout)
  const cartUnavailable = cart.status !== "active";

  const totalWithTip = cart.total_cents + tipCents;

  const handleAddAddress = async () => {
    if (!newAddr.line1.trim()) {
      toast.error("Address line is required");
      return;
    }
    try {
      const created = await createAddr.mutateAsync({
        ...newAddr,
        formatted_address: `${newAddr.line1}, ${newAddr.city}`,
        is_default: (addresses?.length ?? 0) === 0,
      });
      setSelectedAddrId(created.id);
      setShowAddrForm(false);
      toast.success("Address saved");
    } catch {
      toast.error("Failed to save address");
    }
  };

  const handlePlaceOrder = async () => {
    if (!selectedAddrId) {
      toast.error("Please select a delivery address");
      return;
    }
    setSubmitting(true);
    try {
      const result = await placeOrder.mutateAsync({
        address_id: selectedAddrId,
        tip_cents: tipCents,
        notes: notes.trim() || undefined,
      });
      // Detect payment mode from response
      if (result.payment.mock_mode) {
        setPaymentMode("mock");
        toast.success("Order placed! Payment succeeded (mock mode).");
      } else {
        setPaymentMode("stripe");
        toast.success("Order created! Complete payment via Stripe.");
        // In production: call stripe.confirmCardPayment(result.payment.client_secret)
        // For now, the backend marks the intent as pending; webhook will
        // update the order to paid once Stripe confirms.
      }
      clearCart();
      navigate(`/orders/${result.order.id}/track`);
    } catch (e: unknown) {
      const msg =
        (e as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error
          ?.message ?? "Failed to place order";
      toast.error(msg);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <h1 className="font-display text-2xl font-bold mb-6">Checkout</h1>

      {cartUnavailable && (
        <div className="mb-4 rounded-lg bg-red-50 border border-red-200 p-3 text-sm text-red-800">
          This cart is no longer active (status: {cart.status}). The restaurant may have closed or the cart expired.
          Please <button onClick={() => navigate("/")} className="underline font-medium">browse restaurants</button> to start a new order.
        </div>
      )}

      <div className="grid gap-6 md:grid-cols-3">
        {/* Left: address + payment */}
        <div className="md:col-span-2 space-y-6">
          {/* Address */}
          <section className="card p-5">
            <div className="flex items-center justify-between">
              <h2 className="font-semibold flex items-center gap-2">
                <MapPin size={16} /> Delivery Address
              </h2>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowAddrForm((s) => !s)}
              >
                <Plus size={14} /> New
              </Button>
            </div>

            {addrLoading ? (
              <p className="mt-3 text-sm text-gray-500">Loading…</p>
            ) : (
              <div className="mt-3 space-y-2">
                {addresses?.map((a) => (
                  <label
                    key={a.id}
                    className={clsx(
                      "block cursor-pointer rounded-lg border p-3 text-sm transition",
                      selectedAddrId === a.id
                        ? "border-brand-500 bg-brand-50"
                        : "border-gray-200 hover:bg-gray-50",
                    )}
                  >
                    <div className="flex items-start gap-2">
                      <input
                        type="radio"
                        name="address"
                        checked={selectedAddrId === a.id}
                        onChange={() => setSelectedAddrId(a.id)}
                        className="mt-1 accent-brand-500"
                      />
                      <div>
                        <div className="font-medium">
                          {a.label}{" "}
                          {a.is_default && (
                            <span className="text-xs text-gray-500">(default)</span>
                          )}
                        </div>
                        <div className="text-gray-600">
                          {a.line1}
                          {a.line2 ? `, ${a.line2}` : ""}, {a.city}
                        </div>
                      </div>
                    </div>
                  </label>
                ))}
                {addresses?.length === 0 && !showAddrForm && (
                  <p className="text-sm text-gray-500">
                    No saved addresses. Add one to continue.
                  </p>
                )}
              </div>
            )}

            {showAddrForm && (
              <div className="mt-4 space-y-2 border-t border-gray-100 pt-4">
                <Input
                  label="Label"
                  value={newAddr.label}
                  onChange={(e) => setNewAddr((s) => ({ ...s, label: e.target.value }))}
                />
                <Input
                  label="Street address"
                  placeholder="123 Main St, Apt 4B"
                  value={newAddr.line1}
                  onChange={(e) => setNewAddr((s) => ({ ...s, line1: e.target.value }))}
                />
                <Input
                  label="City"
                  value={newAddr.city}
                  onChange={(e) => setNewAddr((s) => ({ ...s, city: e.target.value }))}
                />
                <Button variant="secondary" size="sm" onClick={handleAddAddress}>
                  Save address
                </Button>
              </div>
            )}
          </section>

          {/* Payment */}
          <section className="card p-5">
            <h2 className="font-semibold flex items-center gap-2">
              <CreditCard size={16} /> Payment
            </h2>
            {paymentMode === "mock" ? (
              <>
                <p className="mt-2 text-sm text-gray-600">
                  Running in <strong>mock mode</strong> — payment will auto-succeed without
                  real charges. Set <code>STRIPE_SECRET_KEY</code> in the backend to enable
                  real Stripe payments.
                </p>
                <div className="mt-3 rounded-lg bg-emerald-50 p-3 text-sm text-emerald-700 flex items-center gap-2">
                  <CheckCircle2 size={16} /> Mock card on file · charges simulated
                </div>
              </>
            ) : (
              <>
                <p className="mt-2 text-sm text-gray-600">
                  Secure payment processed by <strong>Stripe</strong>. Your card details
                  never touch our servers — they go directly to Stripe via Stripe.js.
                </p>
                <div className="mt-3 rounded-lg bg-blue-50 p-3 text-sm text-blue-700 flex items-center gap-2">
                  <CreditCard size={16} /> Stripe Elements card form will render here
                </div>
                <p className="mt-2 text-xs text-gray-500">
                  To complete production setup: install <code>@stripe/stripe-js</code>,
                  embed a <code>&lt;CardElement /&gt;</code>, and call
                  <code> stripe.confirmCardPayment(client_secret)</code> after order placement.
                </p>
              </>
            )}
          </section>

          {/* Notes */}
          <section className="card p-5">
            <h2 className="font-semibold flex items-center gap-2 mb-3">
              <Tag size={16} /> Order notes (optional)
            </h2>
            <textarea
              className="input min-h-[60px] resize-none"
              placeholder="e.g., leave at door, ring bell, no contact"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </section>
        </div>

        {/* Right: summary */}
        <div>
          <section className="card sticky top-20 p-5">
            <h2 className="font-semibold mb-3">Order summary</h2>
            <div className="space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Subtotal</span>
                <span>${(cart.subtotal_cents / 100).toFixed(2)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Delivery fee</span>
                <span>${(cart.delivery_fee_cents / 100).toFixed(2)}</span>
              </div>
            </div>

            {/* Tip selector */}
            <div className="mt-4">
              <div className="text-sm font-medium mb-2">Add a tip</div>
              <div className="grid grid-cols-4 gap-1">
                {[0, 200, 400, 600].map((amt) => (
                  <button
                    key={amt}
                    onClick={() => setTipCents(amt)}
                    className={clsx(
                      "rounded-lg border px-2 py-1.5 text-xs font-medium transition",
                      tipCents === amt
                        ? "border-brand-500 bg-brand-50 text-brand-700"
                        : "border-gray-200 hover:bg-gray-50",
                    )}
                  >
                    {amt === 0 ? "None" : `$${(amt / 100).toFixed(0)}`}
                  </button>
                ))}
              </div>
            </div>

            <div className="mt-4 flex justify-between font-semibold border-t border-gray-100 pt-3">
              <span>Total</span>
              <span>${(totalWithTip / 100).toFixed(2)}</span>
            </div>

            <Button
              onClick={handlePlaceOrder}
              disabled={submitting || !selectedAddrId || cartUnavailable}
              className="mt-4 w-full"
              size="lg"
            >
              {submitting ? (
                <>
                  <Loader2 size={16} className="animate-spin" /> Placing order…
                </>
              ) : (
                `Place order · $${(totalWithTip / 100).toFixed(2)}`
              )}
            </Button>
            {!selectedAddrId && (
              <p className="mt-2 text-xs text-amber-700 text-center">
                Select a delivery address to continue.
              </p>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}

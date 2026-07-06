import { useEffect, useMemo, useState } from "react";
import { Plus, Minus, X } from "lucide-react";
import { useCartStore } from "@/store/cartStore";
import type { MenuItem, MenuItemCustomization } from "@/types";
import { Button } from "@/components/ui/Button";
import { clsx } from "clsx";

interface Props {
  item: MenuItem;
  restaurantId: string;
  onClose: () => void;
}

export function AddItemModal({ item, restaurantId, onClose }: Props) {
  const addItem = useCartStore((s) => s.addItem);
  const [quantity, setQuantity] = useState(1);
  const [notes, setNotes] = useState("");
  const [selections, setSelections] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);

  // Initialize selections with default options
  useEffect(() => {
    const init: Record<string, string> = {};
    for (const cust of item.customizations) {
      const def = cust.options.find((o) => o.is_default);
      if (def) init[cust.id] = def.id;
    }
    setSelections(init);
  }, [item]);

  const unitPrice = useMemo(() => {
    let total = item.price_cents;
    for (const cust of item.customizations) {
      const optId = selections[cust.id];
      const opt = cust.options.find((o) => o.id === optId);
      if (opt) total += opt.price_cents;
    }
    return total;
  }, [item, selections]);

  const lineTotal = unitPrice * quantity;

  const handleSelect = (cust: MenuItemCustomization, optionId: string) => {
    setSelections((s) => ({ ...s, [cust.id]: optionId }));
  };

  const handleSubmit = async () => {
    // Validate required customizations
    for (const cust of item.customizations) {
      if (cust.is_required && !selections[cust.id]) {
        alert(`Please select: ${cust.name}`);
        return;
      }
    }
    setSubmitting(true);
    try {
      await addItem({
        restaurant_id: restaurantId,
        menu_item_id: item.id,
        quantity,
        customizations: Object.entries(selections).map(([cust_id, option_id]) => ({
          customization_id: cust_id,
          option_id,
        })),
        notes: notes.trim() || undefined,
      });
      onClose();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-black/40 sm:items-center">
      <div className="card w-full max-w-md max-h-[90vh] overflow-y-auto">
        <div className="sticky top-0 flex items-center justify-between border-b border-gray-100 bg-white px-4 py-3">
          <h2 className="font-display text-lg font-semibold">{item.name}</h2>
          <button onClick={onClose} className="rounded p-1 hover:bg-gray-100">
            <X size={18} />
          </button>
        </div>

        <div className="p-4">
          {item.description && <p className="text-sm text-gray-600">{item.description}</p>}
          {item.image_url && (
            <img
              src={item.image_url}
              alt=""
              className="mt-3 h-40 w-full rounded-lg object-cover"
            />
          )}
          <p className="mt-3 text-lg font-semibold text-brand-700">
            ${(item.price_cents / 100).toFixed(2)}
          </p>

          {/* Customizations */}
          {item.customizations.map((cust) => (
            <div key={cust.id} className="mt-4">
              <div className="flex items-center gap-2">
                <span className="font-medium text-sm">{cust.name}</span>
                {cust.is_required && (
                  <span className="text-xs text-red-500">*required</span>
                )}
              </div>
              <div className="mt-2 space-y-1">
                {cust.options.map((opt) => {
                  const selected = selections[cust.id] === opt.id;
                  return (
                    <label
                      key={opt.id}
                      className={clsx(
                        "flex cursor-pointer items-center justify-between rounded-lg border p-2 text-sm transition",
                        selected
                          ? "border-brand-500 bg-brand-50"
                          : "border-gray-200 hover:bg-gray-50",
                      )}
                    >
                      <span className="flex items-center gap-2">
                        <input
                          type="radio"
                          name={cust.id}
                          checked={selected}
                          onChange={() => handleSelect(cust, opt.id)}
                          className="accent-brand-500"
                        />
                        {opt.name}
                      </span>
                      <span className="text-xs text-gray-500">
                        {opt.price_cents > 0
                          ? `+$${(opt.price_cents / 100).toFixed(2)}`
                          : "included"}
                      </span>
                    </label>
                  );
                })}
              </div>
            </div>
          ))}

          {/* Notes */}
          <div className="mt-4">
            <label className="label">Special instructions (optional)</label>
            <textarea
              className="input min-h-[60px] resize-none"
              placeholder="e.g., no onions, extra spicy"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </div>

          {/* Quantity + add to cart */}
          <div className="mt-5 flex items-center gap-3">
            <div className="flex items-center gap-2">
              <button
                onClick={() => setQuantity((q) => Math.max(1, q - 1))}
                className="grid h-9 w-9 place-items-center rounded-lg border border-gray-200 hover:bg-gray-50"
              >
                <Minus size={16} />
              </button>
              <span className="w-8 text-center font-medium">{quantity}</span>
              <button
                onClick={() => setQuantity((q) => q + 1)}
                className="grid h-9 w-9 place-items-center rounded-lg border border-gray-200 hover:bg-gray-50"
              >
                <Plus size={16} />
              </button>
            </div>
            <Button
              onClick={handleSubmit}
              disabled={submitting}
              className="flex-1"
              size="lg"
            >
              {submitting
                ? "Adding…"
                : `Add · $${(lineTotal / 100).toFixed(2)}`}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

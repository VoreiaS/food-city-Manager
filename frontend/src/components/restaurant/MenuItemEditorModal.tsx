import { useEffect, useRef, useState } from "react";
import { X, Upload, Loader2, ImageIcon } from "lucide-react";
import type { MenuItem } from "@/types";
import {
  useCreateMenuItem,
  useUpdateMenuItem,
  useUploadItemPhoto,
  type CreateItemInput,
} from "@/hooks/useRestaurant";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";

interface Props {
  item: MenuItem | null; // null = creating new
  categoryId: string;
  onClose: () => void;
}

export function MenuItemEditorModal({ item, categoryId, onClose }: Props) {
  const isEdit = !!item;
  const createItem = useCreateMenuItem();
  const updateItem = useUpdateMenuItem();
  const uploadPhoto = useUploadItemPhoto();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [name, setName] = useState(item?.name ?? "");
  const [description, setDescription] = useState(item?.description ?? "");
  const [priceCents, setPriceCents] = useState(item?.price_cents ?? 1000);
  const [isVeg, setIsVeg] = useState(item?.is_veg ?? false);
  const [spiceLevel, setSpiceLevel] = useState(item?.spice_level ?? 0);
  const [imageUrl, setImageUrl] = useState(item?.image_url ?? "");
  const [trackStock, setTrackStock] = useState(false);
  const [stockCount, setStockCount] = useState(0);
  const [submitting, setSubmitting] = useState(false);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    // Lock body scroll
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = "";
    };
  }, []);

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (!isEdit || !item) {
      // For new items, just preview locally (upload after create)
      setImageUrl(URL.createObjectURL(file));
      return;
    }
    setUploading(true);
    try {
      const result = await uploadPhoto.mutateAsync({ id: item.id, file });
      setImageUrl(result.image_url);
    } finally {
      setUploading(false);
    }
  };

  const handleSubmit = async () => {
    if (!name.trim()) {
      alert("Item name is required");
      return;
    }
    if (priceCents < 0) {
      alert("Price cannot be negative");
      return;
    }
    setSubmitting(true);
    try {
      if (isEdit && item) {
        await updateItem.mutateAsync({
          id: item.id,
          input: {
            name,
            description: description || undefined,
            price_cents: priceCents,
            is_veg: isVeg,
            spice_level: spiceLevel,
            stock_count: trackStock ? stockCount : undefined,
            image_url: imageUrl || undefined,
          },
        });
      } else {
        const input: CreateItemInput = {
          category_id: categoryId,
          name,
          description: description || undefined,
          price_cents: priceCents,
          is_veg: isVeg,
          spice_level: spiceLevel,
          track_stock: trackStock,
          stock_count: trackStock ? stockCount : undefined,
          image_url: imageUrl || undefined,
        };
        await createItem.mutateAsync(input);
      }
      onClose();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-black/40 sm:items-center" onClick={onClose}>
      <div
        className="card w-full max-w-lg max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="sticky top-0 flex items-center justify-between border-b border-gray-100 bg-white px-4 py-3">
          <h2 className="font-display text-lg font-semibold">
            {isEdit ? "Edit item" : "New menu item"}
          </h2>
          <button onClick={onClose} className="rounded p-1 hover:bg-gray-100">
            <X size={18} />
          </button>
        </div>

        <div className="p-4 space-y-4">
          {/* Photo */}
          <div>
            <label className="label">Photo</label>
            <div className="flex items-center gap-3">
              <div className="h-20 w-20 shrink-0 rounded-lg bg-gray-100 overflow-hidden flex items-center justify-center">
                {imageUrl ? (
                  <img src={imageUrl} alt="" className="h-full w-full object-cover" />
                ) : (
                  <ImageIcon size={24} className="text-gray-400" />
                )}
              </div>
              <div className="flex-1">
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  className="hidden"
                  onChange={handleFileSelect}
                />
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => fileInputRef.current?.click()}
                  disabled={uploading}
                >
                  {uploading ? (
                    <Loader2 size={14} className="animate-spin" />
                  ) : (
                    <Upload size={14} />
                  )}
                  {uploading ? "Uploading…" : "Upload photo"}
                </Button>
                <p className="mt-1 text-xs text-gray-500">JPG/PNG, max 5MB</p>
              </div>
            </div>
          </div>

          {/* Name */}
          <Input
            label="Name *"
            placeholder="e.g., Margherita Pizza"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />

          {/* Description */}
          <div>
            <label className="label">Description</label>
            <textarea
              className="input min-h-[60px] resize-none"
              placeholder="Brief description of the dish"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          {/* Price */}
          <Input
            label="Price (cents) *"
            type="number"
            min={0}
            value={priceCents}
            onChange={(e) => setPriceCents(parseInt(e.target.value) || 0)}
          />
          <p className="-mt-2 text-xs text-gray-500">
            ${(priceCents / 100).toFixed(2)} displayed to customer
          </p>

          {/* Veg + Spice */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="label">Dietary</label>
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={isVeg}
                  onChange={(e) => setIsVeg(e.target.checked)}
                  className="accent-green-600"
                />
                Vegetarian
              </label>
            </div>
            <div>
              <label className="label">Spice level</label>
              <select
                className="input"
                value={spiceLevel}
                onChange={(e) => setSpiceLevel(parseInt(e.target.value))}
              >
                <option value={0}>None</option>
                <option value={1}>Mild 🌶️</option>
                <option value={2}>Medium 🌶️🌶️</option>
                <option value={3}>Hot 🌶️🌶️🌶️</option>
              </select>
            </div>
          </div>

          {/* Stock tracking */}
          <div className="rounded-lg border border-gray-200 p-3">
            <label className="flex items-center gap-2 text-sm font-medium">
              <input
                type="checkbox"
                checked={trackStock}
                onChange={(e) => setTrackStock(e.target.checked)}
                className="accent-brand-500"
              />
              Track inventory
            </label>
            {trackStock && (
              <div className="mt-2">
                <Input
                  label="Current stock"
                  type="number"
                  min={0}
                  value={stockCount}
                  onChange={(e) => setStockCount(parseInt(e.target.value) || 0)}
                />
                <p className="mt-1 text-xs text-gray-500">
                  Customers can't order more than available stock.
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="sticky bottom-0 flex justify-end gap-2 border-t border-gray-100 bg-white px-4 py-3">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={submitting}>
            {submitting ? (
              <Loader2 size={14} className="animate-spin" />
            ) : isEdit ? (
              "Save changes"
            ) : (
              "Create item"
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}

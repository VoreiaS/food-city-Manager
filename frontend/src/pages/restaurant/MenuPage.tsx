import { useState } from "react";
import {
  useRestaurantMenu,
  useUpdateMenuItem,
  useCreateCategory,
  useDeleteMenuItem,
  useDeleteCategory,
} from "@/hooks/useRestaurant";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Plus, Leaf, Flame, Package, Trash2, Pencil, ChevronDown } from "lucide-react";
import { MenuItemEditorModal } from "@/components/restaurant/MenuItemEditorModal";
import { clsx } from "clsx";
import type { MenuItem } from "@/types";

export function MenuPage() {
  const { data: menu, isLoading } = useRestaurantMenu();
  const updateItem = useUpdateMenuItem();
  const createCategory = useCreateCategory();
  const deleteItem = useDeleteMenuItem();
  const deleteCategory = useDeleteCategory();
  const [newCategory, setNewCategory] = useState("");
  const [editingItem, setEditingItem] = useState<{ item: MenuItem | null; categoryId: string } | null>(null);
  const [collapsedCats, setCollapsedCats] = useState<Set<string>>(new Set());

  if (isLoading) return <div className="p-8 text-center text-gray-500">Loading…</div>;
  if (!menu) return <div className="p-8 text-center text-gray-500">No menu found.</div>;

  const handleAddCategory = async () => {
    if (!newCategory.trim()) return;
    await createCategory.mutateAsync(newCategory.trim());
    setNewCategory("");
  };

  const toggleAvailability = async (itemId: string, inStock: boolean) => {
    await updateItem.mutateAsync({
      id: itemId,
      input: { status: inStock ? "out_of_stock" : "available" },
    });
  };

  const toggleCollapse = (catId: string) => {
    setCollapsedCats((prev) => {
      const next = new Set(prev);
      if (next.has(catId)) {
        next.delete(catId);
      } else {
        next.add(catId);
      }
      return next;
    });
  };

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="font-display text-2xl font-bold">Menu Management</h1>
        <span className="text-sm text-gray-500">
          {menu.categories.reduce((acc, c) => acc + c.items.length, 0)} items
        </span>
      </div>

      {/* Add category */}
      <div className="card p-4 mb-6">
        <h2 className="font-semibold mb-2">Add a category</h2>
        <div className="flex gap-2">
          <Input
            placeholder="e.g., Beverages, Desserts"
            value={newCategory}
            onChange={(e) => setNewCategory(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddCategory()}
          />
          <Button onClick={handleAddCategory} disabled={createCategory.isPending}>
            <Plus size={14} /> Add
          </Button>
        </div>
      </div>

      {/* Categories */}
      <div className="space-y-4">
        {menu.categories.map((cat) => {
          const collapsed = collapsedCats.has(cat.id);
          return (
            <section key={cat.id} className="card overflow-hidden">
              {/* Category header */}
              <div className="flex items-center justify-between bg-gray-50 px-4 py-2">
                <button
                  onClick={() => toggleCollapse(cat.id)}
                  className="flex items-center gap-2 font-semibold text-left"
                >
                  <ChevronDown
                    size={16}
                    className={clsx("transition-transform", collapsed && "-rotate-90")}
                  />
                  {cat.name}
                  <span className="text-xs text-gray-500 font-normal">
                    ({cat.items.length})
                  </span>
                </button>
                <div className="flex items-center gap-1">
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => setEditingItem({ item: null, categoryId: cat.id })}
                  >
                    <Plus size={14} /> Add item
                  </Button>
                  <button
                    onClick={() => {
                      if (
                        cat.items.length === 0 &&
                        confirm(`Delete category "${cat.name}"?`)
                      ) {
                        deleteCategory.mutate(cat.id);
                      } else if (cat.items.length > 0) {
                        alert(
                          "Cannot delete a category with items. Move or delete items first.",
                        );
                      }
                    }}
                    className="rounded p-1.5 text-red-500 hover:bg-red-50"
                    title="Delete category"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>

              {/* Items */}
              {!collapsed && (
                <div className="p-3 space-y-2">
                  {cat.items.map((item) => (
                    <div
                      key={item.id}
                      className={clsx(
                        "flex items-center gap-3 rounded-lg border p-3 transition",
                        item.in_stock
                          ? "border-gray-200"
                          : "border-gray-200 bg-gray-50 opacity-75",
                      )}
                    >
                      {item.image_url ? (
                        <img
                          src={item.image_url}
                          alt=""
                          className="h-14 w-14 shrink-0 rounded-lg object-cover"
                        />
                      ) : (
                        <div className="grid h-14 w-14 shrink-0 place-items-center rounded-lg bg-gray-100 text-gray-400">
                          <Package size={16} />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          {item.is_veg && <Leaf size={12} className="text-green-600 shrink-0" />}
                          {item.spice_level > 0 && (
                            <span className="flex items-center text-red-600 shrink-0">
                              {Array.from({ length: item.spice_level }).map((_, i) => (
                                <Flame key={i} size={10} />
                              ))}
                            </span>
                          )}
                          <span className="font-medium truncate">{item.name}</span>
                          {!item.in_stock && (
                            <span className="rounded-full bg-red-100 px-2 py-0.5 text-xs text-red-700 shrink-0">
                              Out of stock
                            </span>
                          )}
                        </div>
                        <div className="text-xs text-gray-500 truncate">
                          ${(item.price_cents / 100).toFixed(2)}
                          {item.description && ` · ${item.description}`}
                        </div>
                      </div>
                      <div className="flex items-center gap-1 shrink-0">
                        <button
                          onClick={() => toggleAvailability(item.id, item.in_stock)}
                          className={clsx(
                            "rounded px-2 py-1 text-xs font-medium transition",
                            item.in_stock
                              ? "text-amber-700 hover:bg-amber-50"
                              : "text-green-700 hover:bg-green-50",
                          )}
                          title={item.in_stock ? "Mark out of stock" : "Mark available"}
                        >
                          {item.in_stock ? "Mark out" : "Mark in"}
                        </button>
                        <button
                          onClick={() => setEditingItem({ item, categoryId: cat.id })}
                          className="rounded p-1.5 text-gray-500 hover:bg-gray-100"
                          title="Edit item"
                        >
                          <Pencil size={14} />
                        </button>
                        <button
                          onClick={() => {
                            if (confirm(`Delete "${item.name}"? This hides it from the menu.`)) {
                              deleteItem.mutate(item.id);
                            }
                          }}
                          className="rounded p-1.5 text-red-500 hover:bg-red-50"
                          title="Delete item"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>
                  ))}
                  {cat.items.length === 0 && (
                    <button
                      onClick={() => setEditingItem({ item: null, categoryId: cat.id })}
                      className="w-full rounded-lg border-2 border-dashed border-gray-200 py-4 text-sm text-gray-400 hover:border-brand-300 hover:text-brand-500"
                    >
                      <Plus size={16} className="mx-auto mb-1" />
                      Add first item to {cat.name}
                    </button>
                  )}
                </div>
              )}
            </section>
          );
        })}
      </div>

      {/* Editor modal */}
      {editingItem && (
        <MenuItemEditorModal
          item={editingItem.item}
          categoryId={editingItem.categoryId}
          onClose={() => setEditingItem(null)}
        />
      )}
    </div>
  );
}

import { create } from "zustand";
import { toast } from "sonner";
import { cartApi, type AddCartItemInput, type CartResponse } from "@/api/cart.api";

interface CartState {
  cart: CartResponse | null;
  isLoading: boolean;
  isDrawerOpen: boolean;

  fetchCart: () => Promise<void>;
  addItem: (input: AddCartItemInput) => Promise<void>;
  updateItem: (itemId: string, payload: { quantity?: number; notes?: string }) => Promise<void>;
  removeItem: (itemId: string) => Promise<void>;
  clearCart: () => Promise<void>;
  openDrawer: () => void;
  closeDrawer: () => void;
}

export const useCartStore = create<CartState>()((set) => ({
  cart: null,
  isLoading: false,
  isDrawerOpen: false,

  fetchCart: async () => {
    set({ isLoading: true });
    try {
      const cart = await cartApi.get();
      set({ cart, isLoading: false });
    } catch {
      set({ isLoading: false });
      // 404 / null is normal when no cart exists; ignore.
    }
  },

  addItem: async (input) => {
    set({ isLoading: true });
    try {
      const cart = await cartApi.add(input);
      set({ cart, isLoading: false, isDrawerOpen: true });
      toast.success("Added to cart");
    } catch (e: unknown) {
      set({ isLoading: false });
      const msg =
        (e as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error
          ?.message ?? "Failed to add to cart";
      toast.error(msg);
      throw e;
    }
  },

  updateItem: async (itemId, payload) => {
    try {
      const cart = await cartApi.update(itemId, payload);
      set({ cart });
    } catch (e: unknown) {
      const msg =
        (e as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error
          ?.message ?? "Failed to update item";
      toast.error(msg);
    }
  },

  removeItem: async (itemId) => {
    try {
      const cart = await cartApi.remove(itemId);
      set({ cart });
    } catch (e: unknown) {
      const msg =
        (e as { response?: { data?: { error?: { message?: string } } } })?.response?.data?.error
          ?.message ?? "Failed to remove item";
      toast.error(msg);
    }
  },

  clearCart: async () => {
    try {
      await cartApi.clear();
      set({ cart: null });
      toast.success("Cart cleared");
    } catch {
      toast.error("Failed to clear cart");
    }
  },

  openDrawer: () => set({ isDrawerOpen: true }),
  closeDrawer: () => set({ isDrawerOpen: false }),
}));

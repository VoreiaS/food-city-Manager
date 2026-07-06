import { create } from "zustand";
import { persist } from "zustand/middleware";
import { authApi, type LoginInput, type RegisterInput } from "@/api/auth.api";
import type { UserPublic } from "@/types";

interface AuthState {
  user: UserPublic | null;
  accessToken: string | null;
  refreshToken: string | null;
  expiresAt: number | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  login: (input: LoginInput) => Promise<void>;
  register: (input: RegisterInput) => Promise<void>;
  refresh: () => Promise<string | null>;
  logout: () => void;
  setUser: (user: UserPublic) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      accessToken: null,
      refreshToken: null,
      expiresAt: null,
      isAuthenticated: false,
      isLoading: false,

      login: async (input) => {
        set({ isLoading: true });
        try {
          const res = await authApi.login(input);
          set({
            user: res.user,
            accessToken: res.access_token,
            refreshToken: res.refresh_token,
            expiresAt: Date.now() + res.expires_in * 1000,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (e) {
          set({ isLoading: false });
          throw e;
        }
      },

      register: async (input) => {
        set({ isLoading: true });
        try {
          const res = await authApi.register(input);
          set({
            user: res.user,
            accessToken: res.access_token,
            refreshToken: res.refresh_token,
            expiresAt: Date.now() + res.expires_in * 1000,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (e) {
          set({ isLoading: false });
          throw e;
        }
      },

      refresh: async () => {
        const currentRefresh = get().refreshToken;
        if (!currentRefresh) return null;
        try {
          const res = await authApi.refresh(currentRefresh);
          set({
            accessToken: res.access_token,
            refreshToken: res.refresh_token,
            expiresAt: Date.now() + res.expires_in * 1000,
            isAuthenticated: true,
            user: res.user,
          });
          return res.access_token;
        } catch {
          get().logout();
          return null;
        }
      },

      logout: () => {
        set({
          user: null,
          accessToken: null,
          refreshToken: null,
          expiresAt: null,
          isAuthenticated: false,
        });
      },

      setUser: (user) => set({ user }),
    }),
    {
      name: "food-city-auth",
      partialize: (state) => ({
        user: state.user,
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
        expiresAt: state.expiresAt,
        isAuthenticated: state.isAuthenticated,
      }),
    },
  ),
);

// Centralized API client with auth interceptors + refresh handling.
//
// When VITE_API_URL is not set, uses relative paths (same origin).
// This allows the frontend to work behind a reverse proxy (nginx) or
// Vercel rewrites without CORS issues.

import axios, { type AxiosInstance, type InternalAxiosRequestConfig } from "axios";
import { useAuthStore } from "@/store/authStore";

const baseURL = import.meta.env.VITE_API_URL
  ? `${import.meta.env.VITE_API_URL}/api/v1`
  : "/api/v1";

export const apiClient: AxiosInstance = axios.create({
  baseURL,
  headers: { "Content-Type": "application/json" },
  timeout: 15_000,
});

// Attach access token on every request
apiClient.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const token = useAuthStore.getState().accessToken;
  if (token) {
    config.headers.set("Authorization", `Bearer ${token}`);
  }
  return config;
});

// Auto-refresh on 401
let refreshPromise: Promise<string | null> | null = null;

apiClient.interceptors.response.use(
  (res) => res,
  async (error) => {
    const original = error.config;
    if (error.response?.status === 401 && !original._retry) {
      original._retry = true;
      try {
        // Singleton refresh — coalesce parallel 401s into one refresh
        if (!refreshPromise) {
          refreshPromise = useAuthStore.getState().refresh();
        }
        const newToken = await refreshPromise;
        refreshPromise = null;
        if (!newToken) {
          useAuthStore.getState().logout();
          return Promise.reject(error);
        }
        original.headers.set("Authorization", `Bearer ${newToken}`);
        return apiClient(original);
      } catch (e) {
        refreshPromise = null;
        useAuthStore.getState().logout();
        return Promise.reject(e);
      }
    }
    return Promise.reject(error);
  },
);

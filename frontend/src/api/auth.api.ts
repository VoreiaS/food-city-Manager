import { apiClient } from "./client";
import type { AuthResponse, UserPublic } from "@/types";

export interface RegisterInput {
  email: string;
  password: string;
  phone: string;
  full_name: string;
  role?: string;
}

export interface LoginInput {
  email: string;
  password: string;
}

export const authApi = {
  register: async (input: RegisterInput) => {
    const { data } = await apiClient.post<AuthResponse>("/auth/register", {
      role: "customer",
      ...input,
    });
    return data;
  },
  login: async (input: LoginInput) => {
    const { data } = await apiClient.post<AuthResponse>("/auth/login", input);
    return data;
  },
  refresh: async (refreshToken: string) => {
    const { data } = await apiClient.post<AuthResponse>("/auth/refresh", {
      refresh_token: refreshToken,
    });
    return data;
  },
  me: async () => {
    const { data } = await apiClient.get<UserPublic>("/auth/me");
    return data;
  },
};

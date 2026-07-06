import { useAuthStore } from "@/store/authStore";
import { useQuery } from "@tanstack/react-query";
import { authApi } from "@/api/auth.api";

// Re-export the auth store hook for convenience
export { useAuthStore };

/**
 * On mount (if authenticated), refetch the user profile to ensure
 * the persisted store is still valid server-side.
 */
export function useCurrentUser() {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const setUser = useAuthStore((s) => s.setUser);
  return useQuery({
    queryKey: ["auth", "me"],
    queryFn: async () => {
      const user = await authApi.me();
      setUser(user);
      return user;
    },
    enabled: isAuthenticated,
    staleTime: 5 * 60 * 1000,
  });
}

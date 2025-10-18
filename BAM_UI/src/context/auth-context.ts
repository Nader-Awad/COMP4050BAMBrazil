import { createContext, useContext } from "react";
import type { User } from "@types";

export type AuthContextValue = {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;  // hydration flag
  login: (email: string, password: string) => Promise<void> | void;
  logout: () => Promise<void> | void;
};

export const AuthContext = createContext<AuthContextValue | null>(null);

export function useAuthContext() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuthContext must be used within AuthProvider");
  return ctx;
}

import React from "react";
import { useAuth } from "@hooks/useAuth";
import { AuthContext, type AuthContextValue } from "@context/auth-context";

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const value = useAuth();
  // value must include: { user, isAuthenticated, isLoading, login, logout }
  return <AuthContext.Provider value={value as AuthContextValue}>{children}</AuthContext.Provider>;
}

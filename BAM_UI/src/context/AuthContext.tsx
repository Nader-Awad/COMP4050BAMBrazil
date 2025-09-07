import React, { createContext, useContext } from "react";
import { Role, User, useAuth } from "../hooks/useAuth";

type AuthContextValue = {
  user: User;
  setUser: (u: User) => void;
  role: Role;
  setRole: (r: Role) => void;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const value = useAuth();
  return <AuthContext.Provider value={value as AuthContextValue}>{children}</AuthContext.Provider>;
}

export function useAuthContext() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuthContext must be used within AuthProvider");
  return ctx;
}


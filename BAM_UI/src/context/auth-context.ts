import { createContext, useContext } from "react";
import type { Role, User } from "@types";

export type AuthContextValue = {
  user: User;
  setUser: (u: User) => void;
  role: Role;
  setRole: (r: Role) => void;
};

export const AuthContext = createContext<AuthContextValue | null>(null);

export function useAuthContext() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuthContext must be used within AuthProvider");
  return ctx;
}
// Context shape and consumer hook for authentication (user + role). The
// provider component lives in `AuthContext.tsx` to keep that file exporting
// only a React component for Fast Refresh friendliness.

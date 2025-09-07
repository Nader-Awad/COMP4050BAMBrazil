/**
 * AuthProvider component only.
 *
 * Provides the authentication state (user + role) to the subtree by wiring the
 * values returned from `useAuth` into the `AuthContext` provider. The actual
 * context value type, context instance, and consumer hook live in
 * `@context/auth-context` to keep this file exporting a single React component
 * (helps React Fast Refresh and keeps concerns separated).
 */
import React from "react";
import { useAuth } from "@hooks/useAuth";
import { AuthContext, type AuthContextValue } from "@context/auth-context";

/**
 * Mount at the app root (e.g. around <App />) to make auth state available.
 */
export function AuthProvider({ children }: { children: React.ReactNode }) {
  const value = useAuth();
  return <AuthContext.Provider value={value as AuthContextValue}>{children}</AuthContext.Provider>;
}

/**
 * AppProvider component only.
 *
 * Thin provider around a generic AppContext for app-level misc state. The
 * context object and consumer hook live in `@context/app-context` to keep this
 * file exporting a single React component (plays nicely with Fast Refresh).
 */
import React from "react";
import { AppContext, type AppContextValue } from "@context/app-context";

export function AppProvider({ value = {}, children }: { value?: AppContextValue; children: React.ReactNode }) {
  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

import { createContext, useContext } from "react";

export type AppContextValue = Record<string, unknown>;

export const AppContext = createContext<AppContextValue | null>(null);

export function useAppContext<T extends AppContextValue = AppContextValue>() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useAppContext must be used within AppProvider");
  return ctx as T;
}
// Context shape and consumer hook for miscellaneous app-level state shared
// across the UI. Kept separate from the provider component to satisfy Fast
// Refresh constraints (provider file only exports a component).

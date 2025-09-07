import React, { createContext, useContext } from "react";

type AppContextValue = Record<string, unknown>;
const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ value = {}, children }: { value?: AppContextValue; children: React.ReactNode }) {
  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useAppContext<T extends AppContextValue = AppContextValue>() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useAppContext must be used within AppProvider");
  return ctx as T;
}


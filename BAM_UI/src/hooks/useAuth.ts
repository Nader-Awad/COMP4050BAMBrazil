import { useEffect, useRef, useState } from "react";
import type { User } from "@types";
import {
  API_BASE,
  ACCESS_TOKEN_KEY,
  REFRESH_TOKEN_KEY,
  clearTokens,
  setTokens,
} from "@/services/apiClient";

// tiny helper: decode base64url JWT w/o external deps
function decodeJwt<T = any>(jwt: string): T | null {
  try {
    const [, payload] = jwt.split(".");
    if (!payload) return null;
    const b64 = payload.replace(/-/g, "+").replace(/_/g, "/");
    const pad = "=".repeat((4 - (b64.length % 4)) % 4);
    const json = atob(b64 + pad);
    return JSON.parse(json) as T;
  } catch {
    return null;
  }
}

// schedule a callback ~60s before exp
function scheduleRefreshToken(jwt: string, cb: () => void): () => void {
  const now = Math.floor(Date.now() / 1000);
  const exp = decodeJwt<{ exp?: number }>(jwt)?.exp ?? now + 3600;
  const ms = Math.max((exp - 60 - now) * 1000, 5_000);
  const id = setTimeout(cb, ms);
  return () => clearTimeout(id);
}

const STORAGE_USER    = "auth_user";
const STORAGE_REFRESH = REFRESH_TOKEN_KEY;
const STORAGE_ACCESS  = ACCESS_TOKEN_KEY;

function makeAuthUrl(path: string) {
  try {
    return new URL(path, API_BASE).toString();
  } catch {
    const base = API_BASE.endsWith("/") ? API_BASE.slice(0, -1) : API_BASE;
    return `${base}${path}`;
  }
}

type LoginOk = {
  success: boolean;
  message?: string | null;
  error?: string | null;
  data?: {
    expires_in: number;
    refresh_token: string;
    token: string;
    user: { email: string; id: string; name: string; role: string };
  };
};

type RefreshOk = {
  success: boolean;
  message?: string | null;
  error?: string | null;
  data?: { expires_in: number; refresh_token: string; token: string };
};

type Claims = { sub?: string; user_id?: string; uid?: string; name?: string; email?: string; role?: string; exp?: number; [k:string]:any };

function normalizeRole(r?: string): User["role"] | null {
  const v = (r || "").toLowerCase();
  return v === "student" || v === "teacher" || v === "admin" ? v : null;
}

function userFromJwt(jwt: string): User | null {
  const c = decodeJwt<Claims>(jwt);
  if (!c) return null;
  const id   = (c.sub ?? c.user_id ?? c.uid) as string | undefined;
  const role = normalizeRole(c.role || (c as any)?.roles?.[0]);
  const name = (c.name ?? c.email ?? "User") as string;
  if (!id || !role) return null;
  return { id, name, role };
}

function isJwtValid(jwt: string, skew = 10): boolean {
  const c = decodeJwt<Claims>(jwt);
  if (!c?.exp) return false;
  const now = Math.floor(Date.now() / 1000);
  return c.exp > now + skew;
}

export function useAuth() {

  // Initialize from localStorage to avoid wiping tokens on the first effect run
  const [user, setUser] = useState<User | null>(() => {
    const u = localStorage.getItem(STORAGE_USER);
    try { return u ? (JSON.parse(u) as User) : null; } catch { return null; }
  });
  const [accessToken, setAccessToken] = useState<string | null>(() => localStorage.getItem(STORAGE_ACCESS));
  const [refreshToken, setRefreshToken] = useState<string | null>(() => localStorage.getItem(STORAGE_REFRESH));

  const [isLoading, setIsLoading] = useState(true);
  const isAuthenticated = !!user;

  // persist user / tokens (now safe; state is already seeded)
  useEffect(() => {
    if (user) localStorage.setItem(STORAGE_USER, JSON.stringify(user));
    else localStorage.removeItem(STORAGE_USER);
  }, [user]);

  // restore/refresh on mount (do NOT force-logout on refresh hiccups)
  useEffect(() => {
    // If we already have a valid AT, derive user immediately for UX
    if (accessToken && isJwtValid(accessToken)) {
      const u = userFromJwt(accessToken);
      if (u) setUser(u);
    }

    (async () => {
      const rt = refreshToken;
      if (!rt) { setIsLoading(false); return; }
      try {
        const r = await fetch(makeAuthUrl("/api/auth/refresh"), {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ refresh_token: rt }),
        });
        const raw = await r.text().catch(() => "");
        if (!r.ok) { console.warn("[refresh:on-mount] HTTP", r.status, raw); return; }
        let j: RefreshOk | null = null;
        try { j = raw ? JSON.parse(raw) : null; } catch {}
        if (!j || j.success !== true || !j.data || !j.data.token) { console.warn("[refresh:on-mount] bad payload", j || raw); return; }
        const d = j.data;
        setTokens(d.token, d.refresh_token ?? undefined);
        setAccessToken(d.token);
        setRefreshToken(d.refresh_token ?? null);
        const u = userFromJwt(d.token);
        if (u) setUser(u);
      } catch (e) {
        console.warn("[refresh:on-mount] exception", e);
      } finally {
        setIsLoading(false);
      }
    })();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // important: run once

  // auto-refresh ~60s before exp; don't force-logout on failure
  const clearRef = useRef<null | (() => void)>(null);
  useEffect(() => {
    if (clearRef.current) { clearRef.current(); clearRef.current = null; }
    if (accessToken && refreshToken) {
      clearRef.current = scheduleRefreshToken(accessToken, async () => {
        try {
          const r = await fetch(makeAuthUrl("/api/auth/refresh"), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ refresh_token: refreshToken }),
          });
          const raw = await r.text().catch(() => "");
          if (!r.ok) { console.warn("[refresh:auto] HTTP", r.status, raw); return; }
          let j: RefreshOk | null = null;
          try { j = raw ? JSON.parse(raw) : null; } catch {}
          if (!j || j.success !== true || !j.data || !j.data.token) { console.warn("[refresh:auto] bad payload", j || raw); return; }
          const d = j.data;
          setTokens(d.token, d.refresh_token ?? undefined);
          setAccessToken(d.token);
          setRefreshToken(d.refresh_token ?? null);
          const u = userFromJwt(d.token);
          if (u) setUser(u);
        } catch (e) {
          console.warn("[refresh:auto] exception", e);
        }
      });
    }
    return () => { if (clearRef.current) clearRef.current(); };
  }, [accessToken, refreshToken]);

  async function login(email: string, password: string) {
    const res = await fetch(makeAuthUrl("/api/auth/login"), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });
    const raw = await res.text().catch(() => "");
    if (!res.ok) {
      let msg = `Login failed (${res.status}).`;
      try { const j = raw ? (JSON.parse(raw) as LoginOk) : null; msg = (j?.message || j?.error || raw || msg) as string; }
      catch { if (raw) msg = raw; }
      throw new Error(msg);
    }
    let j: LoginOk | null = null;
    try { j = raw ? JSON.parse(raw) : null; } catch {}
    if (!j || j.success !== true || !j.data || !j.data.token) {
      const msg = (j?.message || j?.error || "Login failed.") as string;
      throw new Error(msg);
    }
    const d = j.data;
    setTokens(d.token, d.refresh_token);
    setAccessToken(d.token);
    setRefreshToken(d.refresh_token ?? null);
    const u = userFromJwt(d.token) ?? {
      id: d.user.id,
      name: d.user.name,
      role: (d.user.role || "").toLowerCase() as User["role"],
    };
    setUser(u);
  }

  async function logout() {
    try {
      await fetch(makeAuthUrl("/api/auth/logout"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
      });
    } finally {
      setUser(null);
      setAccessToken(null);
      setRefreshToken(null);
      localStorage.removeItem(STORAGE_USER);
      clearTokens();
    }
  }

  return { user, isAuthenticated, isLoading, login, logout };
}

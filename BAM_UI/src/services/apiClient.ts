/* eslint-disable @typescript-eslint/no-explicit-any */
/**
 * Centralized API client for the BAM Brazil UI (fetch-based).
 *
 * Features:
 * - Environment-based base URL (Vite: VITE_API_URL)
 * - Type-safe methods for core endpoints (bookings, sessions, images, microscope)
 * - JWT access token injection + automatic refresh on 401
 * - Request/response interceptors (pre/post hooks)
 * - Retry with exponential backoff for network/5xx errors
 * - Loading state tracking (subscribe via onLoading / useApiLoading hook)
 * - Friendly error messages (ProblemDetails aware, JSON fallback)
 */

import { useEffect, useState } from "react";
import type { Booking } from "@/types/booking";
import type { User } from "@/types/user";

// ---------- Config ----------

const API_BASE =
  (import.meta.env.VITE_API_URL as string | undefined) ??
  `${window.location.origin}`; // default to same-origin in dev

// If your backend uses a different refresh path, change here:
const REFRESH_ENDPOINT = "/api/auth/refresh";

// Optional: where login/logout live (for clarity only)
const LOGIN_ENDPOINT = "/api/auth/login";
const LOGOUT_ENDPOINT = "/api/auth/logout";

// ---------- Token Store (localStorage) ----------

const ACCESS_TOKEN_KEY = "accessToken";
const REFRESH_TOKEN_KEY = "refreshToken";

function getAccessToken() {
  return localStorage.getItem(ACCESS_TOKEN_KEY);
}
function getRefreshToken() {
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}
function setTokens(access?: string | null, refresh?: string | null) {
  if (access !== undefined) {
    if (access) localStorage.setItem(ACCESS_TOKEN_KEY, access);
    else localStorage.removeItem(ACCESS_TOKEN_KEY);
  }
  if (refresh !== undefined) {
    if (refresh) localStorage.setItem(REFRESH_TOKEN_KEY, refresh);
    else localStorage.removeItem(REFRESH_TOKEN_KEY);
  }
}
function clearTokens() {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}

// ---------- Loading State Emitter ----------

type LoadingListener = (activeCount: number) => void;
const listeners = new Set<LoadingListener>();
let activeRequests = 0;

function notify() {
  for (const fn of listeners) fn(activeRequests);
}
function beginRequest() {
  activeRequests += 1;
  notify();
}
function endRequest() {
  activeRequests = Math.max(0, activeRequests - 1);
  notify();
}

/** Subscribe to loading counter changes. Returns unsubscribe. */
export function onLoading(listener: LoadingListener) {
  listeners.add(listener);
  // initial notify for immediate state
  listener(activeRequests);
  return () => {
    listeners.delete(listener);
  };
}

/** React hook for easy loading state consumption */
export function useApiLoading() {
  const [count, setCount] = useState(activeRequests);
  useEffect(() => onLoading(setCount), []);
  return count > 0;
}

// ---------- Error Handling ----------

export class ApiError extends Error {
  status?: number;
  code?: string;
  details?: unknown;
  constructor(message: string, opts?: { status?: number; code?: string; details?: unknown }) {
    super(message);
    this.name = "ApiError";
    this.status = opts?.status;
    this.code = opts?.code;
    this.details = opts?.details;
  }
}

/** Turn a response body into a friendly message (ProblemDetails or JSON fallback). */
async function parseErrorMessage(res: Response): Promise<{ message: string; code?: string; details?: any }> {
  const ct = res.headers.get("content-type") ?? "";
  try {
    if (ct.includes("application/json")) {
      const data = await res.json();
      // ProblemDetails (RFC 7807) fields or generic JSON
      const title = data.title ?? data.error ?? "Request failed";
      const detail = data.detail ?? data.message ?? "";
      const message = [title, detail].filter(Boolean).join(": ");
      return { message: message || `HTTP ${res.status}`, code: data.code, details: data };
    }
    const text = await res.text();
    return { message: text || `HTTP ${res.status}` };
  } catch {
    return { message: `HTTP ${res.status}` };
  }
}

// ---------- Fetch Wrapper (with interceptors, auth, retry, refresh) ----------

type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
type RequestInitExt = RequestInit & { retry?: number; retryDelayMs?: number };

type PreInterceptor = (input: RequestInfo, init: RequestInit) => Promise<{ input: RequestInfo; init: RequestInit }> | { input: RequestInfo; init: RequestInit };
type PostInterceptor = (response: Response, cloned: Response) => Promise<void> | void;

const preInterceptors: PreInterceptor[] = [];
const postInterceptors: PostInterceptor[] = [];

export function addRequestInterceptor(fn: PreInterceptor) {
  preInterceptors.push(fn);
}
export function addResponseInterceptor(fn: PostInterceptor) {
  postInterceptors.push(fn);
}

/** Core request function used by all endpoint helpers. */
async function request<T>(path: string, method: HttpMethod, body?: any, init?: RequestInitExt): Promise<T> {
  const url = path.startsWith("http") ? path : `${API_BASE}${path}`;
  let headers: HeadersInit = {
    Accept: "application/json",
  };

  if (body !== undefined && !(body instanceof FormData)) {
    headers = { ...headers, "Content-Type": "application/json" };
  }

  const auth = getAccessToken();
  if (auth) headers = { ...headers, Authorization: `Bearer ${auth}` };

  let reqInit: RequestInitExt = {
    method,
    headers,
    credentials: "include", // in case backend also uses httpOnly cookies
    body: body instanceof FormData ? body : body !== undefined ? JSON.stringify(body) : undefined,
    ...init,
  };

  // Run pre-interceptors
  for (const pre of preInterceptors) {
    const res = await pre(url, reqInit);
    url; // no change to url in this design (could support if needed)
    reqInit = res.init as RequestInitExt;
  }

  const maxRetry = init?.retry ?? 2;
  const baseDelay = init?.retryDelayMs ?? 300;

  let attempt = 0;
  let lastErr: unknown;

  beginRequest();
  try {
    while (attempt <= maxRetry) {
      try {
        const res = await fetch(url, reqInit);

        // Clone for post-interceptors so body can still be read outside
        const clone = res.clone();
        for (const post of postInterceptors) {
          await post(res, clone);
        }

        // 401 -> try refresh once then retry original
        if (res.status === 401 && attempt <= maxRetry) {
          const refreshed = await tryRefreshToken();
          if (refreshed) {
            // inject new token and retry immediately
            const newAccess = getAccessToken();
            if (newAccess) {
              (reqInit.headers as Record<string, string>) = {
                ...(reqInit.headers as Record<string, string>),
                Authorization: `Bearer ${newAccess}`,
              };
            }
            attempt++;
            continue;
          }
          // refresh failed -> clear and throw
          clearTokens();
          const { message } = await parseErrorMessage(res);
          throw new ApiError(message || "Unauthorised", { status: 401 });
        }

        // Success path
        if (res.ok) {
          // No content
          if (res.status === 204) return undefined as T;
          const ct = res.headers.get("content-type") ?? "";
          if (ct.includes("application/json")) return (await res.json()) as T;
          // Non-JSON (e.g. images/blob). Cast as any.
          // @ts-expect-error (caller should know the expected type)
          return await res.blob();
        }

        // 5xx -> retry with backoff
        if (res.status >= 500 && res.status <= 599 && attempt < maxRetry) {
          await delay(expBackoff(attempt, baseDelay));
          attempt++;
          continue;
        }

        // Other errors (4xx etc.)
        const { message, code, details } = await parseErrorMessage(res);
        throw new ApiError(message || `HTTP ${res.status}`, { status: res.status, code, details });
      } catch (err: any) {
        lastErr = err;
        // Network / CORS / aborted -> retry
        const transient =
          err instanceof TypeError ||
          (err?.name === "AbortError") ||
          (err?.message && /network|fetch|failed|offline/i.test(String(err.message)));

        if (transient && attempt < maxRetry) {
          await delay(expBackoff(attempt, baseDelay));
          attempt++;
          continue;
        }
        throw err;
      }
    }
    // If we exit the loop, throw the last error
    throw lastErr instanceof Error ? lastErr : new Error("Unknown request error");
  } finally {
    endRequest();
  }
}

function delay(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}
function expBackoff(attempt: number, base: number) {
  // attempt: 0,1,2 -> 1x,2x,4x jitters
  const factor = 2 ** attempt;
  const jitter = Math.random() * 0.25 + 0.875; // 0.875..1.125
  return Math.round(base * factor * jitter);
}

// ---------- JWT Refresh ----------

/**
 * Try to refresh the access token using the refresh token.
 * Expected response shape is flexible; adapt to your backend (shown common patterns below).
 *
 * Example accepted responses:
 *  { accessToken: "new", refreshToken?: "maybe new" }
 * or
 *  { token: "newAccess" }
 */
async function tryRefreshToken(): Promise<boolean> {
  const rt = getRefreshToken();
  if (!rt) return false;

  try {
    const res = await fetch(`${API_BASE}${REFRESH_ENDPOINT}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      credentials: "include",
      body: JSON.stringify({ refreshToken: rt }),
    });

    if (!res.ok) return false;

    const data = await res.json().catch(() => ({}));
    const access = data.accessToken ?? data.token ?? data.access_token;
    const refresh = data.refreshToken ?? data.refresh_token;

    if (access) {
      setTokens(access, refresh ?? undefined);
      return true;
    }
    return false;
  } catch {
    return false;
  }
}

// ---------- Public API: typed endpoint helpers ----------

// Bookings
export const BookingsAPI = {
  list(): Promise<Booking[]> {
    return request<Booking[]>("/api/bookings", "GET");
  },
  get(id: string): Promise<Booking> {
    return request<Booking>(`/api/bookings/${encodeURIComponent(id)}`, "GET");
  },
  create(payload: Partial<Booking>): Promise<Booking> {
    return request<Booking>("/api/bookings", "POST", payload);
  },
  update(id: string, payload: Partial<Booking>): Promise<Booking> {
    return request<Booking>(`/api/bookings/${encodeURIComponent(id)}`, "PUT", payload);
  },
  remove(id: string): Promise<void> {
    return request<void>(`/api/bookings/${encodeURIComponent(id)}`, "DELETE");
  },
};

// Sessions (active usage tracking)
export const SessionsAPI = {
  list(): Promise<any[]> {
    return request<any[]>("/api/sessions", "GET");
  },
  get(id: string): Promise<any> {
    return request<any>(`/api/sessions/${encodeURIComponent(id)}`, "GET");
  },
  // add anything your backend supports (start/stop/heartbeat etc.)
};

// Images
export const ImagesAPI = {
  getImage(id: string): Promise<Blob> {
    // non-JSON response expected -> request<Blob> will return a Blob
    return request<Blob>(`/api/images/${encodeURIComponent(id)}`, "GET");
  },
  getLatestForSession(sessionId: string): Promise<Blob> {
    return request<Blob>(`/api/sessions/${encodeURIComponent(sessionId)}/images/latest`, "GET");
  },
};

// Microscope control
export const MicroscopeAPI = {
  command(microscopeId: string, command: string, params?: Record<string, unknown>): Promise<any> {
    return request<any>(
      `/api/microscope/${encodeURIComponent(microscopeId)}/command`,
      "POST",
      { command, params }
    );
  },
  capture(microscopeId: string): Promise<Blob> {
    return request<Blob>(`/api/microscope/${encodeURIComponent(microscopeId)}/capture`, "POST");
  },
  status(microscopeId: string): Promise<any> {
    return request<any>(`/api/microscope/${encodeURIComponent(microscopeId)}/status`, "GET");
  },
};

// Auth convenience helpers (optional)
export const AuthAPI = {
  async login(credentials: { email: string; password: string }): Promise<{ user: User; accessToken: string; refreshToken?: string }> {
    const res = await request<any>(LOGIN_ENDPOINT, "POST", credentials);
    // normalize tokens regardless of backend field names
    const access = res.accessToken ?? res.token ?? res.access_token;
    const refresh = res.refreshToken ?? res.refresh_token;
    if (access) setTokens(access, refresh ?? undefined);
    return res;
  },
  async logout(): Promise<void> {
    try {
      await request<void>(LOGOUT_ENDPOINT, "POST");
    } finally {
      clearTokens();
    }
  },
  setTokens,
  clearTokens,
  getAccessToken,
  getRefreshToken,
};

// ---------- Friendly error-to-toast mapping (example) ----------

export function humanizeError(err: unknown): string {
  if (err instanceof ApiError) {
    // targeted messages for common statuses
    if (err.status === 401) return "Your session expired. Please sign in again.";
    if (err.status === 403) return "You don’t have permission to perform this action.";
    if (err.status === 404) return "We couldn’t find what you were looking for.";
    if (err.status && err.status >= 500) return "The server hit a snag. Please try again.";
    return err.message || "Something went wrong.";
  }
  if (err instanceof Error) {
    if (/network|offline|fetch/i.test(err.message)) return "Network error. Check your connection and try again.";
    return err.message;
  }
  return "Unexpected error.";
}

// ---------- Example request/response interceptors (optional) ----------

// Add a correlation ID header to every request
addRequestInterceptor(async (input, init) => {
  const hdrs = new Headers(init.headers || {});
  hdrs.set("x-correlation-id", crypto.randomUUID());
  return { input, init: { ...init, headers: hdrs } };
});

// Log 5xx responses (could route to Sentry)
addResponseInterceptor(async (res) => {
  if (res.status >= 500) {
    // eslint-disable-next-line no-console
    console.error("Server error", res.status, res.url);
  }
});
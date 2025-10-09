import { Navigate, Outlet } from "react-router-dom";
import { useAuthContext } from "@context/auth-context";

export default function RequireAuth() {
  const { user, isLoading } = useAuthContext();

  // Temporary: add this inside RequireAuth render
  console.log("RequireAuth:", { isLoading, hasUser: !!user, storedAT: !!localStorage.getItem("auth_access") });

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center text-slate-600">
        <span className="animate-pulse">Loading…</span>
      </div>
    );
  }
  
  if (!user) return <Navigate to="/login" replace />;
  return <Outlet />;
}

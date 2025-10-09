import { Navigate } from "react-router-dom";
import { useAuthContext } from "@context/auth-context";

export default function PostLogin() {
  const { user } = useAuthContext();
  return user ? <Navigate to="/app" replace /> : <Navigate to="/login" replace />;
}

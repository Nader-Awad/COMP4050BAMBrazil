import { Routes, Route, Navigate } from "react-router-dom";
import LoginPage from "./LoginOut/LoginPage";
import RequireAuth from "./LoginOut/RequireAuth";
import BioscopeBookingUI from "./BioscopeBookingUI";
import PostLogin from "./LoginOut/PostLogin";

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      {/* after successful login we route once through here */}
      <Route path="/post-login" element={<PostLogin />} />

      {/* Protected app area */}
      <Route element={<RequireAuth />}>
        <Route path="/app" element={<BioscopeBookingUI />} />
      </Route>

      {/* default → app (which is protected) */}
      <Route path="*" element={<Navigate to="/app" replace />} />
    </Routes>
  );
}

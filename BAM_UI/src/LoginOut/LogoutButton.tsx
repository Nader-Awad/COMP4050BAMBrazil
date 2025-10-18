import { useNavigate } from "react-router-dom";
import { useAuthContext } from "@context/auth-context";

export default function LogoutButton() {
  const { logout } = useAuthContext();
  const nav = useNavigate();

  return (
    <button
      onClick={async () => {
        await logout();
        nav("/login");
      }}
      className="px-3 py-2 rounded-xl border border-slate-200 bg-white text-slate-700 hover:bg-slate-50"
    >
      Log out
    </button>
  );
}

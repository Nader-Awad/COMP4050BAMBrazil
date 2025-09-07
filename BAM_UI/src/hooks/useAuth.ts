// Minimal local auth state hook used by the AuthProvider. Abstracted to keep
// the provider thin and to simplify future replacement with a real auth source.
import { useState } from "react";
import type { Role, User } from "@types";

export function useAuth(initial: User = { id: "u-stu-01", name: "Alex Student", role: "student" }) {
  const [user, setUser] = useState<User>(initial);
  const [role, setRole] = useState<Role>(initial.role);
  return { user, setUser, role, setRole };
}

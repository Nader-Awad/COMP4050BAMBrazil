import { useState } from "react";

export type Role = "student" | "teacher" | "admin";
export type User = { id: string; name: string; role: Role };

export function useAuth(initial: User = { id: "u-stu-01", name: "Alex Student", role: "student" }) {
  const [user, setUser] = useState<User>(initial);
  const [role, setRole] = useState<Role>(initial.role);
  return { user, setUser, role, setRole };
}


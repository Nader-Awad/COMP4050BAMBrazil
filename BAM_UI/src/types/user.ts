export type Role = "student" | "teacher" | "admin";

export type User = {
  id: string;
  name: string;
  email: string;
  role: Role;
  created_at: string;
  updated_at: string;
};


export type UserInfo = {
  id: string;
  name: string;
  email: string;
  role: Role;
}


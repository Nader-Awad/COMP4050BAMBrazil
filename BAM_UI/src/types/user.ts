export type Role = "student" | "teacher" | "admin" ;

export type Size = "sm" | "md" | "lg" | "xl";

export type User = {
  id: string;
  name: string;
  role: Role;
};

export type UserAvatarProps = {
  name?: string;
  src?: string;
  size?: Size;
  alt?: string;
  className?: string;
}


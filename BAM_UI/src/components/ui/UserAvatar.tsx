import React from "react";

export interface UserAvatarProps {
  name?: string;
  src?: string;
  size?: "sm" | "md" | "lg";
  alt?: string;
  className?: string;
}

const sizeMap = {
  sm: "w-8 h-8 text-sm",
  md: "w-10 h-10 text-base",
  lg: "w-12 h-12 text-lg",
};

function initials(name?: string) {
  if (!name) return "?";
  return name
    .split(" ")
    .map((n) => n.charAt(0).toUpperCase())
    .slice(0, 2)
    .join("");
}

export default function UserAvatar({ name, src, size = "md", alt, className }: UserAvatarProps) {
  const sz = sizeMap[size];
  const fallback = initials(name);
  if (src) {
    return (
      <img
        src={src}
        alt={alt ?? name ?? "User avatar"}
        className={`rounded-full object-cover ${sz} ${className ?? ""}`}
      />
    );
  }
  return (
    <div
      className={`rounded-full bg-gray-200 dark:bg-gray-700 flex items-center justify-center ${sz} ${className ?? ""}`}
      role="img"
      aria-label={alt ?? name ?? "User avatar"}
    >
      <span className="text-gray-700 dark:text-gray-100 font-medium">{fallback}</span>
    </div>
  );
}
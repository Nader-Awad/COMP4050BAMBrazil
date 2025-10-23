import React from "react";

export type Status = "ok" | "warning" | "error" | "unknown";

export interface StatusIndicatorProps {
  status: Status;
  label?: string;
  size?: "sm" | "md" | "lg";
  "aria-live"?: "polite" | "assertive" | "off";
}

const sizeMap = {
  sm: "w-2 h-2",
  md: "w-3 h-3",
  lg: "w-4 h-4",
};

const colorMap: Record<Status, string> = {
  ok: "bg-green-500",
  warning: "bg-yellow-400",
  error: "bg-red-500",
  unknown: "bg-gray-400",
};

export default function StatusIndicator({
  status,
  label,
  size = "md",
  "aria-live": ariaLive = "polite",
}: StatusIndicatorProps) {
  const color = colorMap[status] || colorMap.unknown;
  return (
    <div
      className="inline-flex items-center gap-2"
      role="status"
      aria-live={ariaLive}
      aria-label={label ?? `Status: ${status}`}
    >
      <span className={`inline-block rounded-full ${sizeMap[size]} ${color}`} aria-hidden="true" />
      {label ? <span className="text-sm">{label}</span> : null}
    </div>
  );
}
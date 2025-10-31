import React from "react";
import StatusIndicator from "../StatusIndicator";

export default {
  title: "UI/StatusIndicator",
  component: StatusIndicator,
};

export const Default = () => (
  <div className="space-x-4">
    <StatusIndicator status="ok" label="Operational" />
    <StatusIndicator status="warning" label="Degraded" />
    <StatusIndicator status="error" label="Offline" />
  </div>
);
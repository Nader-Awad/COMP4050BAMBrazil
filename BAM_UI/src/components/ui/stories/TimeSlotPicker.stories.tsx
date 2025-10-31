import React, { useState } from "react";
import TimeSlotPicker from "../TimeSlotPicker";

const slots = [
  { id: "1", start: "09:00", end: "09:30", label: "09:00 — 09:30" },
  { id: "2", start: "09:30", end: "10:00", label: "09:30 — 10:00" },
  { id: "3", start: "10:00", end: "10:30", label: "10:00 — 10:30", disabled: true },
];

export default {
  title: "UI/TimeSlotPicker",
  component: TimeSlotPicker,
};

export const Default = () => {
  const [selected, setSelected] = useState<string | null>(null);
  return <TimeSlotPicker slots={slots} selectedId={selected ?? undefined} onChange={(id) => setSelected(id)} />;
};
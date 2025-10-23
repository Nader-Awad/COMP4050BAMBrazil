import React, { type KeyboardEvent } from "react";

export interface TimeSlot {
  id: string;
  start: string; // ISO or human-readable
  end?: string;
  disabled?: boolean;
  label?: string;
}

export interface TimeSlotPickerProps {
  slots: TimeSlot[];
  selectedId?: string | null;
  onChange: (id: string) => void;
  ariaLabel?: string;
}

export default function TimeSlotPicker({ slots, selectedId, onChange, ariaLabel = "Select time slot" }: TimeSlotPickerProps) {
  const currentIndex = slots.findIndex((s) => s.id === selectedId);
  const clampIndex = (i: number) => Math.max(0, Math.min(i, slots.length - 1));

  const handleKey = (e: KeyboardEvent<HTMLUListElement>) => {
    if (e.key === "ArrowDown" || e.key === "ArrowRight") {
      const next = clampIndex((currentIndex === -1 ? 0 : currentIndex) + 1);
      if (!slots[next].disabled) onChange(slots[next].id);
      e.preventDefault();
    } else if (e.key === "ArrowUp" || e.key === "ArrowLeft") {
      const prev = clampIndex((currentIndex === -1 ? 0 : currentIndex) - 1);
      if (!slots[prev].disabled) onChange(slots[prev].id);
      e.preventDefault();
    } else if (e.key === "Home") {
      const first = 0;
      if (!slots[first].disabled) onChange(slots[first].id);
      e.preventDefault();
    } else if (e.key === "End") {
      const last = slots.length - 1;
      if (!slots[last].disabled) onChange(slots[last].id);
      e.preventDefault();
    }
  };

  return (
    <ul
      role="listbox"
      aria-label={ariaLabel}
      tabIndex={0}
      onKeyDown={handleKey}
      className="flex flex-col gap-2"
    >
      {slots.map((slot) => {
        const selected = slot.id === selectedId;
        return (
          <li key={slot.id}>
            <button
              role="option"
              aria-selected={selected}
              aria-disabled={slot.disabled}
              disabled={slot.disabled}
              onClick={() => onChange(slot.id)}
              className={`w-full text-left px-3 py-2 rounded-md focus:outline-none focus:ring-2 ${
                selected ? "bg-indigo-600 text-white" : "bg-white text-gray-800"
              } ${slot.disabled ? "opacity-50 cursor-not-allowed" : "hover:bg-gray-50"}`}
            >
              <div className="flex justify-between">
                <span>{slot.label ?? `${slot.start}${slot.end ? ` — ${slot.end}` : ""}`}</span>
                {selected ? <span className="sr-only">selected</span> : null}
              </div>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
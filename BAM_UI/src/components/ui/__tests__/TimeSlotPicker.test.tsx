import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import TimeSlotPicker from "../TimeSlotPicker";
import { expect, test, vi } from "vitest";

const slots = [
  { id: "1", start: "09:00", label: "09:00" },
  { id: "2", start: "09:30", label: "09:30" },
];

test("selects a slot via click and keyboard", () => {
  const onChange = vi.fn();
  render(<TimeSlotPicker slots={slots} onChange={onChange} />);
  const option = screen.getByText("09:00");
  fireEvent.click(option);
  expect(onChange).toHaveBeenCalledWith("1");
});
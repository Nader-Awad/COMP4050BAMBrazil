import React from "react";
import { render, screen } from "@testing-library/react";
import MicroscopeCard from "../MicroscopeCard";
import { expect, test } from "vitest";

const mic = { id: "m1", name: "Mic A", status: "ok" as any, location: "Lab" };

test("renders microscope name and location", () => {
  render(<MicroscopeCard microscope={mic} />);
  expect(screen.getByText("Mic A")).toBeInTheDocument();
  expect(screen.getByText("Lab")).toBeInTheDocument();
});
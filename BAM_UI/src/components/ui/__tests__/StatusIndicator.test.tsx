import React from "react";
import { render, screen } from "@testing-library/react";
import StatusIndicator from "../StatusIndicator";
import { expect, test } from "vitest";

test("renders status indicator with label and ARIA", () => {
  render(<StatusIndicator status="ok" label="All good" />);
  expect(screen.getByRole("status")).toBeInTheDocument();
  expect(screen.getByText("All good")).toBeInTheDocument();
});
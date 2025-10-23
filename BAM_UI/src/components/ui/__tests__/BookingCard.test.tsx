import React from "react";
import { render, screen } from "@testing-library/react";
import BookingCard from "../BookingCard";
import { expect, test } from "vitest";

const booking = {
  id: "b1",
  userName: "User X",
  microscope: "Scope 1",
  start: "now",
  status: "ok" as any,
  images: [],
};

test("renders booking card with user and microscope", () => {
  render(<BookingCard booking={booking} />);
  expect(screen.getByText("User X")).toBeInTheDocument();
  expect(screen.getByText("Scope 1")).toBeInTheDocument();
});
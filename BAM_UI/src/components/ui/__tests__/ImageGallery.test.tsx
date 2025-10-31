import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import ImageGallery from "../ImageGallery";
import { expect, test } from "vitest";
import "@testing-library/jest-dom";

const images = [
  { src: "https://placekitten.com/200/100", alt: "a" },
  { src: "https://placekitten.com/201/100", alt: "b" },
];

test("renders gallery and navigates with buttons", () => {
  render(<ImageGallery images={images} />);
  const next = screen.getByLabelText(/Next image/i);
  fireEvent.click(next);
  expect(screen.getByAltText("b")).toBeInTheDocument();
});
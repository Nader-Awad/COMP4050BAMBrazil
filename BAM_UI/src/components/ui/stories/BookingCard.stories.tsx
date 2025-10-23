import React from "react";
import BookingCard from "../BookingCard";

const booking = {
  id: "b1",
  userName: "Jane Doe",
  userAvatar: undefined,
  microscope: "Scope A",
  start: "2025-11-01 09:00",
  end: "2025-11-01 09:30",
  status: "ok" as any,
  images: [{ src: "https://placekitten.com/600/300", alt: "img1" }],
};

export default {
  title: "UI/BookingCard",
  component: BookingCard,
};

export const Default = () => <BookingCard booking={booking} onEdit={() => {}} onCancel={() => {}} />;
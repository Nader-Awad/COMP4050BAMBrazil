// Local in-memory booking store with helpers to add/update/remove and a small
// derived index (byDate). Keeps business logic out of components.
import { useMemo, useState } from "react";
import type { Booking } from "@types";

export function useBookings(initial: Booking[] = []) {
  const [bookings, setBookings] = useState<Booking[]>(initial);

  const byDate = useMemo(() => {
    const map: Record<string, Booking[]> = {};
    for (const b of bookings) {
      (map[b.date] ||= []).push(b);
    }
    return map;
  }, [bookings]);

  function add(booking: Booking) {
    setBookings((prev) => [...prev, booking]);
  }
  function updateStatus(id: string, status: Booking["status"]) {
    setBookings((prev) => prev.map((b) => (b.id === id ? { ...b, status } : b)));
  }
  function remove(id: string) {
    setBookings((prev) => prev.filter((b) => b.id !== id));
  }

  return { bookings, setBookings, byDate, add, updateStatus, remove };
}

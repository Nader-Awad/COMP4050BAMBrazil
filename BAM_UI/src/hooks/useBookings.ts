import { useMemo, useState } from "react";

export type Booking = {
  id: string;
  bioscopeId: string;
  date: string;
  slotStart: number;
  slotEnd: number;
  title: string;
  groupName?: string;
  attendees?: number;
  requesterId: string;
  requesterName: string;
  status: "pending" | "approved" | "rejected";
  createdAt: string;
};

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


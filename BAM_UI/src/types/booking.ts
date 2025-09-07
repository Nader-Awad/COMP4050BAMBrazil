export type Slot = { start: number; end: number };

export type Bioscope = { id: string; name: string };

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

export type BookingDraft = {
  title: string;
  groupName: string;
  attendees: number;
  slot: string; // key: `${start}-${end}`
};

export type BookingFilters = {
  showApproved: boolean;
  showPending: boolean;
};


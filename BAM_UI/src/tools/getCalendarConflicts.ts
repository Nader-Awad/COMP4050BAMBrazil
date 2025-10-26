import type { Booking } from "@/types";

export type ConflictGroup = {
  bioscopeId: string;
  date: string;
  startSlot: number;
  endSlot: number;
  bookingIds: string[];
};

/**
 * Detect overlapping bookings per bioscope/date.
 * Assumes slot values are minutes-from-midnight, half-open intervals [start, end).
 * Rejected bookings are ignored by default.
 */
export function getCalendarConflicts(
  bookings: Booking[],
  opts?: { ignoreStatuses?: Booking["status"][] }
): ConflictGroup[] {
  const ignore = new Set(opts?.ignoreStatuses ?? ["rejected"]);
  const byKey = new Map<string, Booking[]>();

  for (const b of bookings) {
    if (ignore.has(b.status)) continue;
    const k = `${b.bioscopeId}|${b.date}`;
    if (!byKey.has(k)) byKey.set(k, []);
    byKey.get(k)!.push(b);
  }

  const groups: ConflictGroup[] = [];
  for (const [k, list] of byKey.entries()) {
    const [bioscopeId, date] = k.split("|");
    const sorted = list.slice().sort((a, b) => a.slotStart - b.slotStart || a.slotEnd - b.slotEnd);

    let cur: Booking[] = [];
    let curEnd = -1;

    for (const b of sorted) {
      const overlaps = cur.length > 0 && b.slotStart < curEnd; // [start,end)
      if (!overlaps) {
        if (cur.length > 1) {
          groups.push({
            bioscopeId,
            date,
            startSlot: Math.min(...cur.map(x => x.slotStart)),
            endSlot: Math.max(...cur.map(x => x.slotEnd)),
            bookingIds: cur.map(x => x.id),
          });
        }
        cur = [b];
        curEnd = b.slotEnd;
      } else {
        cur.push(b);
        curEnd = Math.max(curEnd, b.slotEnd);
      }
    }

    if (cur.length > 1) {
      groups.push({
        bioscopeId,
        date,
        startSlot: Math.min(...cur.map(x => x.slotStart)),
        endSlot: Math.max(...cur.map(x => x.slotEnd)),
        bookingIds: cur.map(x => x.id),
      });
    }
  }
  return groups;
}

import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";
import { Badge } from "@components/ui/badge";
import { ShieldCheck } from "lucide-react";
import type { Booking } from "@types";
import type { ConflictGroup } from "@/tools/getCalendarConflicts";

/* ------------------ Helpers ------------------ */
function ymdToLocalDate(ymd: string) {
  const [y, m, d] = ymd.split("-").map(Number);
  return new Date(y, (m ?? 1) - 1, d ?? 1);
}
function fmtDate(d: Date, opts?: Intl.DateTimeFormatOptions) {
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    ...opts,
  }).format(d);
}

/* ------------------ Conflict Checker ------------------ */
function isInConflict(id: string, groups?: ConflictGroup[]) {
  return !!groups?.some((g) => g.bookingIds.includes(id));
}

/* ------------------ Component ------------------ */
export default function BookingSchedule({
  title,
  bookings,
  fmtTime,
  conflicts,
}: {
  title: string;
  bookings: Booking[];
  fmtTime: (minutesFromMidnight: number) => string;
  conflicts?: ConflictGroup[];
}) {
  return (
    <Card className="shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base">
          <ShieldCheck className="w-4 h-4" /> {title}
        </CardTitle>
      </CardHeader>

      <CardContent className="space-y-3">
        {bookings.length === 0 ? (
          <div className="p-6 rounded-xl bg-slate-50 text-slate-600">
            No bookings yet.
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {bookings.map((b) => {
              const conflicted = isInConflict(b.id, conflicts);
              return (
                <div
                  key={b.id}
                  role="group"
                  tabIndex={0}
                  aria-label={`Booking for ${b.title} by ${b.requesterName}`}
                  className={[
                    "relative rounded-2xl border p-3 bg-white shadow-sm transition focus:outline-none focus:ring-2 focus:ring-blue-400",
                    b.status === "pending"
                      ? "border-amber-300"
                      : "border-slate-300",
                    conflicted ? "ring-2 ring-red-300" : "",
                  ].join(" ")}
                >
                  <div className="flex items-center justify-between">
                    <div className="text-sm font-medium">
                      {fmtDate(ymdToLocalDate(b.date))} ·{" "}
                      {fmtTime(b.slotStart)} – {fmtTime(b.slotEnd)}
                    </div>

                    {conflicted && (
                      <span
                        role="img"
                        aria-label="Time conflict detected"
                        title="This booking overlaps another in the same time slot"
                        className="ml-2 text-red-500 text-xs font-semibold cursor-help"
                      >
                        ⚠
                      </span>
                    )}
                  </div>

                  <div className="text-slate-700">{b.title}</div>
                  <div className="text-xs text-slate-500">
                    Requested by {b.requesterName}
                  </div>

                  {b.groupName && (
                    <div className="mt-1 text-xs text-slate-600">
                      Group: {b.groupName} · {b.attendees ?? 1} attendee(s)
                    </div>
                  )}

                  <div className="mt-2">
                    <Badge
                      variant={
                        b.status === "pending" ? "secondary" : "default"
                      }
                    >
                      {b.status}
                    </Badge>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

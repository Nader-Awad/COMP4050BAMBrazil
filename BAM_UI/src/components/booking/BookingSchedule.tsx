// Student view: list of bookings for the selected date/bioscope, rendered with
// status badges and requester details.
import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";
import { Badge } from "@components/ui/badge";
import { ShieldCheck } from "lucide-react";
import type { Booking } from "@types";

export default function BookingSchedule({
  title,
  bookings,
  fmtTime,
}: {
  title: string;
  bookings: Booking[];
  fmtTime: (minutesFromMidnight: number) => string;
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
          <div className="p-6 rounded-xl bg-slate-50 text-slate-600">No bookings yet.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {bookings.map((b) => (
              <div key={b.id} className={`rounded-2xl border p-3 bg-white shadow-sm ${b.status === "pending" ? "border-amber-300" : "border-emerald-300"}`}>
                <div className="text-sm font-medium">{fmtTime(b.slotStart)} – {fmtTime(b.slotEnd)}</div>
                <div className="text-slate-700">{b.title}</div>
                <div className="text-xs text-slate-500">Requested by {b.requesterName}</div>
                {b.groupName && (
                  <div className="mt-1 text-xs text-slate-600">Group: {b.groupName} · {b.attendees ?? 1} attendee(s)</div>
                )}
                <div className="mt-2">
                  <Badge variant={b.status === "pending" ? "secondary" : "default"}>{b.status}</Badge>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

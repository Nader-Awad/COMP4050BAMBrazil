// Teacher view: compact timeline for a single day/bioscope showing whether
// each slot is open or which booking occupies it.
import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";
import { Clock } from "lucide-react";
import type { Slot, Booking } from "@types";

export default function DayAtAGlance({
  slotsForDay,
  dayBookings,
  fmtTime,
}: {
  slotsForDay: Slot[];
  dayBookings: Booking[];
  fmtTime: (minutesFromMidnight: number) => string;
}) {
  return (
    <Card className="shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base"><Clock className="w-4 h-4" /> Day at a glance</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {slotsForDay.map((s) => {
            const match = dayBookings.find((b) => b.slotStart === s.start);
            let slotClass = "flex items-center justify-between rounded-xl border p-2 text-sm ";
            if (match) {
              slotClass += match.status === "approved"
                ? "border-emerald-300 bg-emerald-50"
                : "border-amber-300 bg-amber-50";
            } else {
              slotClass += "border-slate-200 bg-white";
            }
            return (
              <div key={s.start} className={slotClass}>
                <div className="font-medium">{fmtTime(s.start)} – {fmtTime(s.end)}</div>
                <div className="text-slate-600">
                  {match ? (
                    <span>{match.title} · <span className="text-slate-500">{match.requesterName}</span></span>
                  ) : (
                    <span className="text-slate-400">Open</span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}

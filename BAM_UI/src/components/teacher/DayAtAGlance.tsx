// Teacher view: compact timeline for a single day/bioscope showing whether
// each slot is open or which booking occupies it.
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";
import { Clock, Trash2 } from "lucide-react";
import { Button } from "@components/ui/button";
import type { Slot, Booking } from "@types";

export default function DayAtAGlance({
  slotsForDay,
  dayBookings,
  fmtTime,
  onDelete,
}: {
  slotsForDay: Slot[];
  dayBookings: Booking[];
  fmtTime: (minutesFromMidnight: number) => string;
  onDelete?: (id: string) => void;
}) {
  return (
    <Card className="shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base">
          <Clock className="w-4 h-4" /> Day at a glance
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {slotsForDay.map((s) => {
            const match = dayBookings.find((b) => b.slotStart <= s.start && b.slotEnd > s.start);
            let slotClass = "flex items-center justify-between rounded-xl border p-2 text-sm ";
            if (match) {
              slotClass += match.status === "approved"
                ? "border-emerald-300 bg-emerald-50"
                : "border-amber-300 bg-amber-50";
            } else {
              slotClass += "border-slate-200 bg-white";
            }
            const timeLabel = `${fmtTime(s.start)} – ${fmtTime(s.end)}`;

            const handleDelete = () => {
              if (!onDelete || !match) return;
              const ok = window.confirm(
                `Delete approved booking #${match.id} (${timeLabel})? This frees the slot.`
              );
              if (ok) onDelete(match.id);
            };

            return (
              <div key={s.start} className={slotClass}>
                <div className="font-medium">{timeLabel}</div>
                <div className="flex items-center gap-2 text-slate-600">
                  {match ? (
                    <>
                      <span>
                        {match.title} · <span className="text-slate-500">{match.requesterName}</span>
                      </span>

                      {onDelete && match.status === "approved" && (
                        <Button size="sm" variant="secondary" onClick={handleDelete} className="shrink-0">
                          <Trash2 className="w-4 h-4 mr-1" />
                          Delete
                        </Button>
                      )}
                    </>
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

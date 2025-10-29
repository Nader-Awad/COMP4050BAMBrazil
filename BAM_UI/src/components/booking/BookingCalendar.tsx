import { motion } from "framer-motion";
import { CalendarClock, Clock } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";

type Slot = { start: number; end: number };

type BookingCalendarProps = {
  openSlots: Slot[];
  slotMinutes: number;
  selectedSlot: string;
  onSelectSlot: (slotKey: string) => void;
  fmtTime: (minutesFromMidnight: number) => string;
  conflictBands?: { start: number; end: number }[];
  pendingBands?: { start: number; end: number }[];
  disableConflictedSelection?: boolean;
};

function overlaps(a: { start: number; end: number }, b: { start: number; end: number }) {
  return a.start < b.end && b.start < a.end;
}

export default function BookingCalendar({
  openSlots,
  slotMinutes,
  selectedSlot,
  onSelectSlot,
  fmtTime,
  conflictBands,
  pendingBands,
  disableConflictedSelection,
}: BookingCalendarProps) {
  return (
    <Card className="lg:col-span-2 shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base">
          <CalendarClock className="w-4 h-4" /> Open slots
        </CardTitle>
      </CardHeader>
      <CardContent>
        {openSlots.length === 0 ? (
          <div className="p-6 rounded-xl bg-amber-50 text-amber-700 flex items-center gap-3">
            No open slots for this date/bioscope.
          </div>
        ) : (
          <div className="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-3">
            {openSlots.map((s) => {
              const slotKey = `${s.start}-${s.end}`;
              const isApprovedBlocked =
                !!conflictBands?.some((band) => overlaps({ start: s.start, end: s.end }, band));
              const isPendingReserved =
                !!pendingBands?.some((band) => overlaps({ start: s.start, end: s.end }, band));

              const disabled = !!disableConflictedSelection && isApprovedBlocked;
              const title =
                isApprovedBlocked
                  ? "Booked (approved)"
                  : isPendingReserved
                  ? "Requested (pending approval)"
                  : undefined;

              return (
                <motion.button
                  key={slotKey}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  disabled={disabled}
                  title={title}
                  onClick={() => {
                    if (disabled) return;
                    onSelectSlot(slotKey);
                  }}
                  className={[
                    "text-left rounded-2xl border p-3 shadow-sm transition",
                    selectedSlot === slotKey ? "border-slate-900" : "border-slate-200",
                    isApprovedBlocked
                      ? "bg-red-50 text-red-700 border-red-200 cursor-not-allowed"
                      : isPendingReserved
                      ? "bg-amber-50 text-amber-800 border-amber-200"
                      : "",
                  ].join(" ")}
                >
                  <div className="text-sm font-medium">
                    {fmtTime(s.start)} – {fmtTime(s.end)}
                  </div>
                  <div className="text-xs text-slate-500 flex items-center gap-1">
                    <Clock className="w-3 h-3" /> {slotMinutes} min
                  </div>
                </motion.button>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

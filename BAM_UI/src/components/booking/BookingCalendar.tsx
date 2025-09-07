import React from "react";
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
};

// Grid of open slots for a day/bioscope with simple selection behavior.
export default function BookingCalendar({ openSlots, slotMinutes, selectedSlot, onSelectSlot, fmtTime }: BookingCalendarProps) {
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
              return (
                <motion.button
                  key={slotKey}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={() => onSelectSlot(slotKey)}
                  className={`text-left rounded-2xl border p-3 shadow-sm transition ${selectedSlot === slotKey ? "border-slate-900" : "border-slate-200"}`}
                >
                  <div className="text-sm font-medium">{fmtTime(s.start)} – {fmtTime(s.end)}</div>
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

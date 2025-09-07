/**
 * Teacher view: pending approvals list with conflict/fair-use hints.
 * Accepts the full bookings list to compute conflicts and requester stats.
 */
import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";
import { Badge } from "@components/ui/badge";
import { Button } from "@components/ui/button";
import { Check, ShieldCheck, X } from "lucide-react";
import type { Booking, Bioscope } from "@types";

export default function ApprovalQueue({
  bookings,
  bioscopes,
  fmtTime,
  setStatus,
}: {
  bookings: Booking[];
  bioscopes: Bioscope[];
  fmtTime: (minutesFromMidnight: number) => string;
  setStatus: (id: string, status: Booking["status"]) => void;
}) {
  const pending = bookings.filter((b) => b.status === "pending").sort((a, b) => a.date.localeCompare(b.date) || a.slotStart - b.slotStart);

  return (
    <Card className="lg:col-span-2 shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base"><ShieldCheck className="w-4 h-4" /> Approval queue</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {pending.length === 0 ? (
          <div className="p-6 rounded-xl bg-slate-50 text-slate-600">No pending requests.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {pending.map((b) => {
              const conflicts = bookings.filter((o) => o.id !== b.id && o.bioscopeId === b.bioscopeId && o.date === b.date && !(b.slotEnd <= o.slotStart || b.slotStart >= o.slotEnd));
              const sameRequester = bookings.filter((o) => o.requesterId === b.requesterId && o.status === "approved").length;
              const bioscopeName = bioscopes.find((x) => x.id === b.bioscopeId)?.name;
              return (
                <div key={b.id} className="rounded-2xl border p-3 bg-white shadow-sm">
                  <div className="flex items-center justify-between">
                    <div className="text-sm font-medium">{b.date} · {fmtTime(b.slotStart)}–{fmtTime(b.slotEnd)}</div>
                    {bioscopeName && <Badge>{bioscopeName}</Badge>}
                  </div>
                  <div className="text-slate-700">{b.title}</div>
                  <div className="text-xs text-slate-500">Requested by {b.requesterName}</div>
                  {b.groupName && (
                    <div className="mt-1 text-xs text-slate-600">Group: {b.groupName} · {b.attendees ?? 1} attendee(s)</div>
                  )}
                  {conflicts.length > 0 && (
                    <div className="mt-2 text-xs text-amber-700 bg-amber-50 p-2 rounded-lg">Potential conflict: {conflicts.length} overlapping booking(s)</div>
                  )}
                  {sameRequester >= 2 && (
                    <div className="mt-2 text-xs text-sky-700 bg-sky-50 p-2 rounded-lg">Fair use hint: requester already has {sameRequester} approved booking(s).</div>
                  )}
                  <div className="mt-3 flex gap-2">
                    <Button size="sm" onClick={() => setStatus(b.id, "approved")}><Check className="w-4 h-4 mr-1" /> Approve</Button>
                    <Button size="sm" variant="destructive" onClick={() => setStatus(b.id, "rejected")}><X className="w-4 h-4 mr-1" /> Reject</Button>
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

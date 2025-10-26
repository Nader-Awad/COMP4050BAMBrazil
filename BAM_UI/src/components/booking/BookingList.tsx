import type { Booking, Bioscope } from "@types";
import { Button } from "@components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@components/ui/card";
import { Badge } from "@components/ui/badge";

type BookingListProps = {
  title?: string;
  bookings: Booking[];
  bioscopes: Bioscope[];
  onCancel?: (id: string) => void; 
  fmtTime: (minutesFromMidnight: number) => string;
};

export default function BookingList({
  title = "My requests",
  bookings,
  bioscopes,
  onCancel,
  fmtTime,
}: BookingListProps) {
  const handleCancel = (b: Booking) => {
    if (!onCancel) return;
    const ok = window.confirm(
      `Cancel your ${b.status} booking #${b.id} on ${b.date} ${fmtTime(b.slotStart)}–${fmtTime(
        b.slotEnd
      )}?`
    );
    if (ok) onCancel(b.id);
  };

  const handleDelete = (b: Booking) => {
    if (!onCancel) return;
    const ok = window.confirm(
      `Delete your approved booking #${b.id} on ${b.date} ${fmtTime(b.slotStart)}–${fmtTime(
        b.slotEnd
      )}? This will remove it permanently and free the slot.`
    );
    if (ok) onCancel(b.id);
  };

  return (
    <Card className="shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="text-base">{title}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {bookings.length === 0 ? (
          <div className="p-6 rounded-xl bg-slate-50 text-slate-600">No bookings.</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
            {bookings.map((b) => {
              let badgeVariant: "default" | "secondary" | "destructive";
              if (b.status === "approved") badgeVariant = "default";
              else if (b.status === "pending") badgeVariant = "secondary";
              else badgeVariant = "destructive";

              const bioscopeName = bioscopes.find((x) => x.id === b.bioscopeId)?.name;

              return (
                <div key={b.id} className="rounded-2xl border p-3 bg-white shadow-sm">
                  <div className="text-sm font-medium">
                    {b.date} · {fmtTime(b.slotStart)}–{fmtTime(b.slotEnd)}
                  </div>
                  <div className="text-slate-700">{b.title}</div>
                  {bioscopeName && <div className="text-xs text-slate-500">{bioscopeName}</div>}

                  <div className="mt-2">
                    <Badge variant={badgeVariant}>{b.status}</Badge>
                  </div>

                  {onCancel && (
                    <div className="mt-2 flex gap-2">
                      {b.status === "approved" ? (
                        <Button
                          variant="destructive"
                          className="text-xs"
                          onClick={() => handleDelete(b)}
                        >
                          Delete
                        </Button>
                      ) : (
                        
                        <Button
                          variant="outline"
                          className="text-xs"
                          onClick={() => handleCancel(b)}
                        >
                          Cancel
                        </Button>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

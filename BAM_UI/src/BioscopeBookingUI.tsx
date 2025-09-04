import React, { useMemo, useState } from "react";
import { motion } from "framer-motion";
import { CalendarClock, Check, Clock, Filter, LineChart, Plus, Users, X, AlertTriangle, Microscope, ShieldCheck, BarChart3 } from "lucide-react";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Badge } from "./components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./components/ui/select";
import { Separator } from "./components/ui/separator";
import { Switch } from "./components/ui/switch";
import {
  ResponsiveContainer,
  AreaChart,
  Area,
  CartesianGrid,
  XAxis,
  YAxis,
  Tooltip,
  BarChart,
  Bar,
} from "recharts";

const BIOSCOPES = [
  { id: "bio-1", name: "Bioscope A" },
  { id: "bio-2", name: "Bioscope B" },
  { id: "bio-3", name: "Bioscope C" },
];

const SCHOOL_HOURS = { start: 8, end: 17 };
const SLOT_MINUTES = 30;

function* range(start: number, end: number, step: number) {
  for (let v = start; v < end; v += step) yield v;
}

function toISODate(d: Date) {
  const z = new Date(d.getTime());
  z.setHours(0, 0, 0, 0);
  return z.toISOString().slice(0, 10);
}

function fmtTime(minutesFromMidnight: number) {
  const h = Math.floor(minutesFromMidnight / 60);
  const m = minutesFromMidnight % 60;
  const ampm = h >= 12 ? "PM" : "AM";
  const hh = ((h + 11) % 12) + 1;
  return `${hh}:${m.toString().padStart(2, "0")} ${ampm}`;
}

function makeDaySlots() {
  const start = SCHOOL_HOURS.start * 60;
  const end = SCHOOL_HOURS.end * 60;
  return Array.from(range(start, end, SLOT_MINUTES));
}

const DAY_SLOTS = makeDaySlots();

type Booking = {
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

export default function BioscopeBookingUI() {
  const [role, setRole] = useState<"student" | "teacher" | "admin">("student");
  const [user] = useState({ id: "u-stu-01", name: "Alex Student", role: "student" });
  const [selectedDate, setSelectedDate] = useState<string>(toISODate(new Date()));
  const [selectedBioscope, setSelectedBioscope] = useState<string>(BIOSCOPES[0].id);

  const [bookings, setBookings] = useState<Booking[]>([
    {
      id: "b1",
      bioscopeId: "bio-1",
      date: toISODate(new Date()),
      slotStart: 9 * 60,
      slotEnd: 10 * 60,
      title: "Yr10 Biology: Cell Observation",
      groupName: "Team Mito",
      attendees: 4,
      requesterId: "u-stu-99",
      requesterName: "Jamie",
      status: "approved",
      createdAt: new Date().toISOString(),
    },
    {
      id: "b2",
      bioscopeId: "bio-1",
      date: toISODate(new Date()),
      slotStart: 13 * 60,
      slotEnd: 13 * 60 + 30,
      title: "Microbe Prep",
      requesterId: "u-stu-88",
      requesterName: "Morgan",
      status: "pending",
      createdAt: new Date().toISOString(),
    },
    {
      id: "b3",
      bioscopeId: "bio-2",
      date: toISODate(new Date()),
      slotStart: 10 * 60 + 30,
      slotEnd: 11 * 60 + 30,
      title: "Chem Bio cross-lab",
      requesterId: "u-stu-77",
      requesterName: "Taylor",
      status: "approved",
      createdAt: new Date().toISOString(),
    },
  ]);

  const [filters, setFilters] = useState({
    showApproved: true,
    showPending: true,
  });

  const slotsForDay = useMemo(() => DAY_SLOTS.map((s) => ({ start: s, end: s + SLOT_MINUTES })), []);

  const dayBookings = useMemo(() => {
    return bookings.filter((b) => b.date === selectedDate && b.bioscopeId === selectedBioscope);
  }, [bookings, selectedDate, selectedBioscope]);

  const occupiedKey = (b: Booking) => `${b.slotStart}-${b.slotEnd}`;

  const occupied = useMemo(() => new Set(dayBookings.map(occupiedKey)), [dayBookings]);

  const openSlots = useMemo(() =>
    slotsForDay.filter((s) => !Array.from(occupied).includes(`${s.start}-${s.end}`)),
    [slotsForDay, occupied]);

  const myBookings = useMemo(() => bookings.filter((b) => b.requesterId === user.id), [bookings, user.id]);

  const [draft, setDraft] = useState({ title: "", groupName: "", attendees: 1, slot: "" });
  const [isGroup, setIsGroup] = useState(false);

  function submitBooking() {
    if (!draft.title || !draft.slot) return;
    const [startStr, endStr] = draft.slot.split("-");
    const start = parseInt(startStr, 10);
    const end = parseInt(endStr, 10);

    const conflict = bookings.some(
      (b) => b.bioscopeId === selectedBioscope && b.date === selectedDate && !(end <= b.slotStart || start >= b.slotEnd)
    );
    if (conflict) {
      alert("This time overlaps an existing booking.");
      return;
    }

    const newBooking: Booking = {
      id: `b-${Math.random().toString(36).slice(2, 8)}`,
      bioscopeId: selectedBioscope,
      date: selectedDate,
      slotStart: start,
      slotEnd: end,
      title: draft.title.trim(),
      groupName: isGroup ? draft.groupName.trim() || undefined : undefined,
      attendees: isGroup ? Math.max(1, Number(draft.attendees) || 1) : undefined,
      requesterId: user.id,
      requesterName: user.name,
      status: "pending",
      createdAt: new Date().toISOString(),
    };
    setBookings((prev) => [...prev, newBooking]);
    setDraft({ title: "", groupName: "", attendees: 1, slot: "" });
  }

  function setStatus(id: string, status: Booking["status"]) {
    setBookings((prev) => prev.map((b) => (b.id === id ? { ...b, status } : b)));
  }

  function removeBooking(id: string) {
    setBookings((prev) => prev.filter((b) => b.id !== id));
  }

  const analytics = useMemo(() => {
    const byDay: Record<string, number> = {};
    const byHour: Record<number, number> = {};
    const byUser: Record<string, number> = {};
    const utilizationByBioscope: Record<string, number> = {};

    const minutesPerDay = (SCHOOL_HOURS.end - SCHOOL_HOURS.start) * 60;

    bookings.forEach((b) => {
      byDay[b.date] = (byDay[b.date] || 0) + (b.slotEnd - b.slotStart);
      const hour = Math.floor(b.slotStart / 60);
      byHour[hour] = (byHour[hour] || 0) + 1;
      byUser[b.requesterName] = (byUser[b.requesterName] || 0) + 1;
      utilizationByBioscope[b.bioscopeId] = (utilizationByBioscope[b.bioscopeId] || 0) + (b.slotEnd - b.slotStart);
    });

    const daySeries = Object.entries(byDay)
      .sort(([a], [b]) => (a < b ? -1 : 1))
      .map(([date, mins]) => ({ date, utilization: Math.round((mins / minutesPerDay) * 100) }));

    const hourSeries = Array.from(range(SCHOOL_HOURS.start, SCHOOL_HOURS.end, 1)).map((h) => ({
      hour: `${h}:00`,
      bookings: byHour[h] || 0,
    }));

    const userSeries = Object.entries(byUser)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([name, count]) => ({ name, count }));

    const bioscopeSeries = BIOSCOPES.map((b) => ({
      name: b.name,
      utilization: Math.round(((utilizationByBioscope[b.id] || 0) / minutesPerDay) * 100),
    }));

    return { daySeries, hourSeries, userSeries, bioscopeSeries };
  }, [bookings]);

  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-50 to-white p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        <header className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
          <div className="flex items-center gap-3">
            <motion.div initial={{ scale: 0.8, opacity: 0 }} animate={{ scale: 1, opacity: 1 }}>
              <div className="p-3 rounded-2xl bg-slate-900 text-white shadow-lg"><Microscope className="w-6 h-6" /></div>
            </motion.div>
            <div>
              <h1 className="text-2xl font-semibold tracking-tight">Bioscope Booking</h1>
              <p className="text-slate-500 text-sm">Reserve lab bioscopes, avoid conflicts, and keep things fair.</p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Select value={role} onValueChange={(v: "student" | "teacher" | "admin") => setRole(v)}>
              <SelectTrigger className="w-[165px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="student">Student view</SelectItem>
                <SelectItem value="teacher">Teacher view</SelectItem>
                <SelectItem value="admin">Admin view</SelectItem>
              </SelectContent>
            </Select>
            <Separator orientation="vertical" className="h-8" />
            <div className="hidden md:flex items-center gap-2 text-sm text-slate-500">
              <Users className="w-4 h-4" />
              <span>{user.name}</span>
            </div>
          </div>
        </header>

        <Card className="border-0 shadow-sm">
          <CardContent className="p-4 md:p-6">
            <div className="flex flex-col md:flex-row gap-4 md:items-end">
              <div className="flex-1">
                <Label className="text-slate-600">Date</Label>
                <Input type="date" value={selectedDate} onChange={(e) => setSelectedDate(e.target.value)} className="mt-1" />
              </div>
              <div className="w-full md:w-[240px]">
                <Label className="text-slate-600">Bioscope</Label>
                <Select value={selectedBioscope} onValueChange={(v: string) => setSelectedBioscope(v)}>
                  <SelectTrigger className="mt-1">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {BIOSCOPES.map((b) => (
                      <SelectItem key={b.id} value={b.id}>{b.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center gap-3 rounded-2xl bg-slate-50 p-3">
                <Filter className="w-4 h-4 text-slate-500" />
                <div className="flex items-center gap-2">
                  <Switch checked={filters.showApproved} onCheckedChange={(v: boolean) => setFilters((f) => ({ ...f, showApproved: v }))} />
                  <span className="text-sm text-slate-600">Approved</span>
                </div>
                <div className="flex items-center gap-2">
                  <Switch checked={filters.showPending} onCheckedChange={(v: boolean) => setFilters((f) => ({ ...f, showPending: v }))} />
                  <span className="text-sm text-slate-600">Pending</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Tabs defaultValue="student" value={role} onValueChange={(v: string) => setRole(v as "student" | "teacher" | "admin")} className="space-y-6">
          <TabsList className="grid grid-cols-3 gap-2 bg-slate-100 p-1 rounded-2xl">
            <TabsTrigger value="student" className="rounded-xl data-[state=active]:bg-white data-[state=active]:shadow">Student</TabsTrigger>
            <TabsTrigger value="teacher" className="rounded-xl data-[state=active]:bg-white data-[state=active]:shadow">Teacher</TabsTrigger>
            <TabsTrigger value="admin" className="rounded-xl data-[state=active]:bg-white data-[state=active]:shadow">Admin</TabsTrigger>
          </TabsList>

          <TabsContent value="student" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <Card className="lg:col-span-2 shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="flex items-center gap-2 text-base"><CalendarClock className="w-4 h-4" /> Open slots</CardTitle>
                </CardHeader>
                <CardContent>
                  {openSlots.length === 0 ? (
                    <div className="p-6 rounded-xl bg-amber-50 text-amber-700 flex items-center gap-3"><AlertTriangle className="w-5 h-5" /> No open slots for this date/bioscope.</div>
                  ) : (
                    <div className="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-3">
                      {openSlots.map((s) => {
                        const slotKey = `${s.start}-${s.end}`;
                        return (
                          <motion.button
                            key={slotKey}
                            whileHover={{ scale: 1.02 }}
                            whileTap={{ scale: 0.98 }}
                            onClick={() => setDraft((d) => ({ ...d, slot: slotKey }))}
                            className={`text-left rounded-2xl border p-3 shadow-sm transition ${draft.slot === slotKey ? "border-slate-900" : "border-slate-200"}`}
                          >
                            <div className="text-sm font-medium">{fmtTime(s.start)} – {fmtTime(s.end)}</div>
                            <div className="text-xs text-slate-500 flex items-center gap-1"><Clock className="w-3 h-3" /> {SLOT_MINUTES} min</div>
                          </motion.button>
                        );
                      })}
                    </div>
                  )}
                </CardContent>
              </Card>

              <Card className="shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="flex items-center gap-2 text-base"><Plus className="w-4 h-4" /> New booking</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div>
                    <Label>Title</Label>
                    <Input value={draft.title} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDraft((d) => ({ ...d, title: e.target.value }))} placeholder="e.g. Yr11 Lab: Enzyme Activity" />
                  </div>
                  <div className="flex items-center justify-between rounded-xl bg-slate-50 p-2">
                    <div className="flex items-center gap-2">
                      <Users className="w-4 h-4 text-slate-600" />
                      <span className="text-sm">Group booking</span>
                    </div>
                    <Switch checked={isGroup} onCheckedChange={setIsGroup} />
                  </div>
                  {isGroup && (
                    <motion.div initial={{ opacity: 0, y: -6 }} animate={{ opacity: 1, y: 0 }} className="grid grid-cols-1 md:grid-cols-2 gap-3">
                      <div>
                        <Label>Group / Team name</Label>
                        <Input value={draft.groupName} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDraft((d) => ({ ...d, groupName: e.target.value }))} placeholder="e.g. Team Enzyme" />
                      </div>
                      <div>
                        <Label>Attendees</Label>
                        <Input type="number" min={1} value={draft.attendees} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDraft((d) => ({ ...d, attendees: Number(e.target.value) }))} />
                      </div>
                    </motion.div>
                  )}
                  <div>
                    <Label>Selected slot</Label>
                    <div className="mt-1 rounded-xl border p-3 text-sm text-slate-700 bg-white">
                      {draft.slot ? (() => { const [a, b] = draft.slot.split("-").map(Number); return `${fmtTime(a)} – ${fmtTime(b)}`; })() : "Pick a slot from the left"}
                    </div>
                  </div>
                  <Button className="w-full" onClick={submitBooking} disabled={!draft.title || !draft.slot}>Request booking</Button>
                  <p className="text-xs text-slate-500">Requests go to teachers for approval to ensure fair use.</p>
                </CardContent>
              </Card>
            </div>

            <Card className="shadow-sm">
              <CardHeader className="pb-2">
                <CardTitle className="flex items-center gap-2 text-base"><ShieldCheck className="w-4 h-4" /> Schedule for {selectedDate} · {BIOSCOPES.find(b => b.id === selectedBioscope)?.name}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {dayBookings.filter((b) => (b.status === "approved" && filters.showApproved) || (b.status === "pending" && filters.showPending)).length === 0 ? (
                  <div className="p-6 rounded-xl bg-slate-50 text-slate-600">No bookings yet.</div>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
                    {dayBookings
                      .filter((b) => (b.status === "approved" && filters.showApproved) || (b.status === "pending" && filters.showPending))
                      .sort((a, b) => a.slotStart - b.slotStart)
                      .map((b) => (
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

            <Card className="shadow-sm">
              <CardHeader className="pb-2"><CardTitle className="text-base">My requests</CardTitle></CardHeader>
              <CardContent className="space-y-3">
                {myBookings.length === 0 ? (
                  <div className="p-6 rounded-xl bg-slate-50 text-slate-600">You haven't requested any bookings yet.</div>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
                    {(() => {
                      const sortedMyBookings = [...myBookings].sort(
                        (a: Booking, b: Booking) =>
                          a.date.localeCompare(b.date) || a.slotStart - b.slotStart
                      );
                      return sortedMyBookings.map((b) => {
                        let badgeVariant: "default" | "secondary" | "destructive";
                        if (b.status === "approved") {
                          badgeVariant = "default";
                        } else if (b.status === "pending") {
                          badgeVariant = "secondary";
                        } else {
                          badgeVariant = "destructive";
                        }
                        return (
                          <div key={b.id} className="rounded-2xl border p-3 bg-white shadow-sm">
                            <div className="text-sm font-medium">{b.date} · {fmtTime(b.slotStart)}–{fmtTime(b.slotEnd)}</div>
                            <div className="text-slate-700">{b.title}</div>
                            <div className="text-xs text-slate-500">{BIOSCOPES.find(x => x.id === b.bioscopeId)?.name}</div>
                            <div className="mt-2"><Badge variant={badgeVariant}>{b.status}</Badge></div>
                            {b.status !== "approved" && (
                              <div className="mt-2 flex gap-2">
                                <Button variant="outline" className="text-xs" onClick={() => removeBooking(b.id)}>Cancel</Button>
                              </div>
                            )}
                          </div>
                        );
                      });
                    })()}
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="teacher" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <Card className="lg:col-span-2 shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="flex items-center gap-2 text-base"><ShieldCheck className="w-4 h-4" /> Approval queue</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  {bookings.filter((b) => b.status === "pending").length === 0 ? (
                    <div className="p-6 rounded-xl bg-slate-50 text-slate-600">No pending requests.</div>
                  ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                      {bookings
                        .filter((b) => b.status === "pending")
                        .sort((a, b) => a.date.localeCompare(b.date) || a.slotStart - b.slotStart)
                        .map((b) => {
                          const conflicts = bookings.filter((o) => o.id !== b.id && o.bioscopeId === b.bioscopeId && o.date === b.date && !(b.slotEnd <= o.slotStart || b.slotStart >= o.slotEnd));
                          const sameRequester = bookings.filter((o) => o.requesterId === b.requesterId && o.status === "approved").length;
                          return (
                            <div key={b.id} className="rounded-2xl border p-3 bg-white shadow-sm">
                              <div className="flex items-center justify-between">
                                <div className="text-sm font-medium">{b.date} · {fmtTime(b.slotStart)}–{fmtTime(b.slotEnd)}</div>
                                <Badge>{BIOSCOPES.find(x => x.id === b.bioscopeId)?.name}</Badge>
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
            </div>
          </TabsContent>

          <TabsContent value="admin" className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <Card className="lg:col-span-2 shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="flex items-center gap-2 text-base"><LineChart className="w-4 h-4" /> Utilization by day</CardTitle>
                </CardHeader>
                <CardContent className="h-[280px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={analytics.daySeries} margin={{ top: 10, right: 10, bottom: 0, left: 0 }}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="date" />
                      <YAxis unit="%" domain={[0, 100]} />
                      <Tooltip />
                      <Area type="monotone" dataKey="utilization" stroke="#8884d8" fill="#8884d8" fillOpacity={0.3} />
                    </AreaChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              <Card className="shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="flex items-center gap-2 text-base"><BarChart3 className="w-4 h-4" /> Demand by hour</CardTitle>
                </CardHeader>
                <CardContent className="h-[280px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={analytics.hourSeries}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="hour" />
                      <YAxis allowDecimals={false} />
                      <Tooltip />
                      <Bar dataKey="bookings" fill="#82ca9d" />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <Card className="shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="text-base">Top requesters</CardTitle>
                </CardHeader>
                <CardContent className="h-[280px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={analytics.userSeries}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis allowDecimals={false} />
                      <Tooltip />
                      <Bar dataKey="count" fill="#8884d8" />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>

              <Card className="lg:col-span-2 shadow-sm">
                <CardHeader className="pb-2">
                  <CardTitle className="text-base">Utilization by bioscope</CardTitle>
                </CardHeader>
                <CardContent className="h-[280px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={analytics.bioscopeSeries}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="name" />
                      <YAxis unit="%" domain={[0, 100]} />
                      <Tooltip />
                      <Bar dataKey="utilization" fill="#82ca9d" />
                    </BarChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </Tabs>

        <p className="text-xs text-slate-500 text-center">This is a demo UI with in-memory state. Wire up to your backend to persist bookings, enforce policies (e.g., per-student limits), and sync across users.</p>
      </div>
    </div>
  );
}
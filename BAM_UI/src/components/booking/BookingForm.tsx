import React from "react";
import { Plus } from "lucide-react";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Switch } from "../../components/ui/switch";

type Draft = { title: string; groupName: string; attendees: number | string; slot: string };

type BookingFormProps = {
  draft: Draft;
  setDraft: React.Dispatch<React.SetStateAction<Draft>>;
  isGroup: boolean;
  setIsGroup: (v: boolean) => void;
  onSubmit: () => void;
  fmtTime: (minutesFromMidnight: number) => string;
};

export default function BookingForm({ draft, setDraft, isGroup, setIsGroup, onSubmit, fmtTime }: BookingFormProps) {
  return (
    <Card className="shadow-sm">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base"><Plus className="w-4 h-4" /> New booking</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div>
          <Label>Title</Label>
          <Input value={draft.title} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDraft((d) => ({ ...d, title: e.target.value }))} placeholder="e.g., Yr10 Biology: Cell Observation" />
        </div>
        <div className="flex items-center gap-2">
          <Switch checked={isGroup} onCheckedChange={(v: boolean) => setIsGroup(Boolean(v))} />
          <span className="text-sm text-slate-600">Group booking</span>
        </div>
        {isGroup && (
          <div className="grid grid-cols-2 gap-2">
            <div>
              <Label>Group name</Label>
              <Input value={draft.groupName} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDraft((d) => ({ ...d, groupName: e.target.value }))} placeholder="e.g., Team Mito" />
            </div>
            <div>
              <Label>Attendees</Label>
              <Input type="number" min={1} value={draft.attendees} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setDraft((d) => ({ ...d, attendees: e.target.value }))} />
            </div>
          </div>
        )}
        <div>
          <Label>Selected slot</Label>
          <div className="mt-1 rounded-xl border p-3 text-sm text-slate-700 bg-white">
            {draft.slot ? (() => { const [a, b] = draft.slot.split("-").map(Number); return `${fmtTime(a)} – ${fmtTime(b)}`; })() : "Pick a slot from the left"}
          </div>
        </div>
        <Button className="w-full" onClick={onSubmit} disabled={!draft.title || !draft.slot}>Request booking</Button>
        <p className="text-xs text-slate-500">Requests go to teachers for approval to ensure fair use.</p>
      </CardContent>
    </Card>
  );
}


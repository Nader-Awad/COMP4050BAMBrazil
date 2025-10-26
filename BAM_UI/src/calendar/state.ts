import {
  startOfDay, endOfDay,
  startOfWeek, endOfWeek,
  addDays, addWeeks
} from "date-fns";

export type ViewMode = "day" | "week";

export function getRange(anchor: Date, mode: ViewMode, weekStartsOn: 0|1|2|3|4|5|6 = 1) {
  if (mode === "day") return { start: startOfDay(anchor), end: endOfDay(anchor) };
  return { start: startOfWeek(anchor, { weekStartsOn }), end: endOfWeek(anchor, { weekStartsOn }) };
}

export function shiftAnchor(anchor: Date, mode: ViewMode, dir: 1 | -1) {
  return mode === "day" ? addDays(anchor, dir) : addWeeks(anchor, dir);
}

export interface ViewState {
  mode: ViewMode;
  anchor: Date;
  visibleStart: Date;
  visibleEnd: Date;
}

export function initView(anchor = new Date(), mode: ViewMode = "week", weekStartsOn: 0|1|2|3|4|5|6 = 1): ViewState {
  const { start, end } = getRange(anchor, mode, weekStartsOn);
  return { mode, anchor, visibleStart: start, visibleEnd: end };
}

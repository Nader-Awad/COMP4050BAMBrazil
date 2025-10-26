import { useCallback, useMemo, useState } from "react";
import { getRange, initView, shiftAnchor, type ViewMode, type ViewState } from "@/calendar/state";

type OnRangeChange = (args: { start: Date; end: Date; mode: ViewMode; anchor: Date }) => void;

export function useCalendarNav(opts?: {
  initialDate?: Date;
  initialMode?: ViewMode;
  weekStartsOn?: 0|1|2|3|4|5|6;
  onRangeChange?: OnRangeChange;
}) {
  const weekStartsOn = opts?.weekStartsOn ?? 1;

  const [state, setState] = useState<ViewState>(() =>
    initView(opts?.initialDate ?? new Date(), opts?.initialMode ?? "week", weekStartsOn)
  );

  const update = useCallback((anchor: Date, mode: ViewMode) => {
    const { start, end } = getRange(anchor, mode, weekStartsOn);
    const next = { mode, anchor, visibleStart: start, visibleEnd: end } as ViewState;
    setState(next);
    opts?.onRangeChange?.({ start, end, mode, anchor });
  }, [opts, weekStartsOn]);

  const setMode = useCallback((mode: ViewMode) => update(state.anchor, mode), [state.anchor, update]);
  const goPrev = useCallback(() => update(shiftAnchor(state.anchor, state.mode, -1), state.mode), [state, update]);
  const goNext = useCallback(() => update(shiftAnchor(state.anchor, state.mode, +1), state.mode), [state, update]);
  const jumpTo = useCallback((date: Date) => update(date, state.mode), [state.mode, update]);
  const goToday = useCallback(() => update(new Date(), state.mode), [state.mode, update]);

  return useMemo(() => ({
    ...state, setMode, goPrev, goNext, jumpTo, goToday
  }), [state, setMode, goPrev, goNext, jumpTo, goToday]);
}

import { useEffect } from "react";
import type { ViewMode } from "./state";
import { Button } from "@components/ui/button";
import { Input } from "@components/ui/input";

type Props = {
  mode: ViewMode;
  onModeChange: (m: ViewMode) => void;
  onPrev: () => void;
  onNext: () => void;
  onToday: () => void;
  rangeLabel: string;
  jumpYMD: string;
  onJumpYMDChange: (ymd: string) => void;
  onJumpCommit: () => void;
};

export function CalendarNav({
  mode,
  onModeChange,
  onPrev,
  onNext,
  onToday,
  rangeLabel,
  jumpYMD,
  onJumpYMDChange,
  onJumpCommit,
}: Props) {
  /* ----------------- Keyboard Navigation ----------------- */
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowLeft") onPrev();
      if (e.key === "ArrowRight") onNext();
      if (e.key.toLowerCase() === "t") onToday();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onPrev, onNext, onToday]);

  return (
    <div
      className="flex flex-wrap items-center gap-2"
      role="navigation"
      aria-label="Calendar navigation"
    >
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant={mode === "day" ? "default" : "outline"}
          className="h-9"
          aria-label="Switch to day view"
          onClick={() => onModeChange("day")}
        >
          Day
        </Button>
        <Button
          type="button"
          variant={mode === "week" ? "default" : "outline"}
          className="h-9"
          aria-label="Switch to week view"
          onClick={() => onModeChange("week")}
        >
          Week
        </Button>
      </div>

      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          className="h-9 px-3"
          aria-label="Previous range"
          onClick={onPrev}
        >
          ‹
        </Button>
        <Button
          type="button"
          variant="outline"
          className="h-9"
          aria-label="Jump to today"
          onClick={onToday}
        >
          Today
        </Button>
        <Button
          type="button"
          variant="outline"
          className="h-9 px-3"
          aria-label="Next range"
          onClick={onNext}
        >
          ›
        </Button>
      </div>

      <div className="flex items-center gap-2">
        <span className="text-sm text-slate-600">Jump to:</span>
        <Input
          type="date"
          className="h-9 w-[160px]"
          aria-label="Select date to jump to"
          value={jumpYMD}
          onChange={(e) => onJumpYMDChange(e.target.value)}
          onBlur={onJumpCommit}
          onKeyDown={(e) => {
            if (e.key === "Enter") (e.target as HTMLInputElement).blur();
          }}
        />
      </div>

      <div
        className="whitespace-nowrap text-slate-600 text-sm ml-2"
        aria-live="polite"
      >
        {rangeLabel}
      </div>
    </div>
  );
}

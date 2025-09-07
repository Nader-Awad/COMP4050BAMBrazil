// Holds available bioscopes and the current selection. Keeps microscope
// concerns isolated from view components and ready for backend wiring.
import { useState } from "react";
import type { Bioscope } from "@types";

export function useMicroscope(initialList: Bioscope[] = [
  { id: "bio-1", name: "Bioscope A" },
  { id: "bio-2", name: "Bioscope B" },
  { id: "bio-3", name: "Bioscope C" },
]) {
  const [bioscopes] = useState<Bioscope[]>(initialList);
  const [selectedBioscope, setSelectedBioscope] = useState<string>(initialList[0]?.id ?? "");
  return { bioscopes, selectedBioscope, setSelectedBioscope };
}

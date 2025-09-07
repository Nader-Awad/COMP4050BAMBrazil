import { useState } from "react";

export type Bioscope = { id: string; name: string };

export function useMicroscope(initialList: Bioscope[] = [
  { id: "bio-1", name: "Bioscope A" },
  { id: "bio-2", name: "Bioscope B" },
  { id: "bio-3", name: "Bioscope C" },
]) {
  const [bioscopes] = useState<Bioscope[]>(initialList);
  const [selectedBioscope, setSelectedBioscope] = useState<string>(initialList[0]?.id ?? "");
  return { bioscopes, selectedBioscope, setSelectedBioscope };
}


import type { Status } from "@/components/ui/StatusIndicator";
import type { GalleryImage } from "./images";

export type Microscope = {
  id: string;
  name: string;
  status: Status;
  location?: string;
  uptime?: string;
  images?: GalleryImage[];
}

export type MicroscopeCardProps = {
  microscope: Microscope;
  onOpen?: (id: string) => void;
  className?: string;
}
import React from "react";
import StatusIndicator, { type Status } from "./StatusIndicator";
import ImageGallery, { type GalleryImage } from "./ImageGallery";

export interface Microscope {
  id: string;
  name: string;
  status: Status;
  location?: string;
  uptime?: string;
  images?: GalleryImage[];
}

export interface MicroscopeCardProps {
  microscope: Microscope;
  onOpen?: (id: string) => void;
  className?: string;
}

export default function MicroscopeCard({ microscope, onOpen, className }: MicroscopeCardProps) {
  return (
    <div className={`p-4 bg-white rounded-md shadow-sm ${className ?? ""}`} aria-labelledby={`mic-${microscope.id}`}>
      <div className="flex items-start gap-4">
        <div className="flex-1">
          <h4 id={`mic-${microscope.id}`} className="font-semibold">{microscope.name}</h4>
          <p className="text-sm text-gray-600">{microscope.location}</p>
          <p className="text-xs text-gray-500">Uptime: {microscope.uptime ?? "N/A"}</p>
        </div>
        <div className="flex flex-col items-end gap-2">
          <StatusIndicator status={microscope.status} label={microscope.status} />
          {onOpen ? (
            <button onClick={() => onOpen(microscope.id)} className="text-sm text-indigo-600 hover:underline">Open</button>
          ) : null}
        </div>
      </div>
      {microscope.images && microscope.images.length > 0 ? (
        <div className="mt-4">
          <ImageGallery images={microscope.images} showThumbnails initialIndex={0} />
        </div>
      ) : null}
    </div>
  );
}
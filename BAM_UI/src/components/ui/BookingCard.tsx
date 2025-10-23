import React from "react";
import StatusIndicator, { type Status } from "./StatusIndicator";
import UserAvatar from "./UserAvatar";
import ImageGallery, { type GalleryImage } from "./ImageGallery";

export interface Booking {
  id: string;
  userName?: string;
  userAvatar?: string;
  microscope?: string;
  start: string;
  end?: string;
  status: Status;
  images?: GalleryImage[];
}

export interface BookingCardProps {
  booking: Booking;
  onEdit?: (id: string) => void;
  onCancel?: (id: string) => void;
  className?: string;
}

export default function BookingCard({ booking, onEdit, onCancel, className }: BookingCardProps) {
  return (
    <article className={`p-4 bg-white rounded-md shadow-sm ${className ?? ""}`} aria-labelledby={`booking-${booking.id}-title`}>
      <header className="flex items-start gap-3">
        <UserAvatar name={booking.userName} src={booking.userAvatar} size="md" />
        <div className="flex-1">
          <h3 id={`booking-${booking.id}-title`} className="font-medium">
            {booking.userName ?? "Unknown user"}
          </h3>
          <p className="text-sm text-gray-600">{booking.microscope ?? "Microscope unknown"}</p>
          <p className="text-xs text-gray-500">{booking.start}{booking.end ? ` — ${booking.end}` : ""}</p>
        </div>
        <div className="flex flex-col items-end gap-2">
          <StatusIndicator status={booking.status} aria-live="polite" />
          <div className="flex gap-2">
            {onEdit ? (
              <button onClick={() => onEdit(booking.id)} className="text-sm text-indigo-600 hover:underline">Edit</button>
            ) : null}
            {onCancel ? (
              <button onClick={() => onCancel(booking.id)} className="text-sm text-red-600 hover:underline">Cancel</button>
            ) : null}
          </div>
        </div>
      </header>
      {booking.images && booking.images.length > 0 ? (
        <div className="mt-4">
          <ImageGallery images={booking.images} showThumbnails />
        </div>
      ) : null}
    </article>
  );
}
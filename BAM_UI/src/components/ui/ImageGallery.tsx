import React, { useState, useCallback, useEffect, type KeyboardEvent } from "react";
import type { GalleryImage, ImageGalleryProps } from "@/types/images";

export default function ImageGallery({
  images,
  initialIndex = 0,
  onIndexChange,
  showThumbnails = true,
  ariaLabel = "Image gallery",
}: Readonly<ImageGalleryProps>) {
  const [index, setIndex] = useState(Math.min(initialIndex, Math.max(0, images.length - 1)));

  useEffect(() => {
    onIndexChange?.(index);
  }, [index, onIndexChange]);

  const prev = useCallback(() => setIndex((i) => (i - 1 + images.length) % images.length), [images.length]);
  const next = useCallback(() => setIndex((i) => (i + 1) % images.length), [images.length]);

  const onKey = (e: KeyboardEvent) => {
    if (e.key === "ArrowLeft") prev();
    if (e.key === "ArrowRight") next();
  };

  if (!images || images.length === 0) return null;

  return (
    <div className="w-full" aria-label={ariaLabel}>
      <div
        className="relative flex items-center justify-center bg-gray-50 rounded-md overflow-hidden"
        role="region"
        aria-label={`${ariaLabel} viewport`}
        tabIndex={0}
        onKeyDown={onKey}
        onClick={() => {}}
        onTouchStart={() => {}}
      >
        <button
          aria-label="Previous image"
          onClick={prev}
          className="absolute left-2 p-1 text-gray-600 hover:text-gray-900 focus:outline-none"
        >
          ‹
        </button>
        <img src={images[index].src} alt={images[index].alt ?? `Image ${index + 1}`} className="max-h-80 object-contain" />
        <button
          aria-label="Next image"
          onClick={next}
          className="absolute right-2 p-1 text-gray-600 hover:text-gray-900 focus:outline-none"
        >
          ›
        </button>
      </div>
      {images[index].caption ? <p className="text-sm mt-2 text-gray-600">{images[index].caption}</p> : null}
      {showThumbnails && images.length > 1 ? (
        <div role="list" className="flex gap-2 mt-3 overflow-x-auto" aria-label="Thumbnails">
          {images.map((img, i) => (
            <button
              key={img.id ?? img.src}
              role="listitem"
              aria-label={`View image ${i + 1}`}
              onClick={() => setIndex(i)}
              className={`flex-shrink-0 rounded border ${i === index ? "ring-2 ring-indigo-400" : "border-transparent"} focus:outline-none`}
            >
              <img src={img.src} alt={img.alt ?? `Thumb ${i + 1}`} className="w-20 h-14 object-cover" />
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
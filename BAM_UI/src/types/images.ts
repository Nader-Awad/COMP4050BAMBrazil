export type GalleryImage = {
  src: string;
  alt?: string;
  caption?: string;
  id?: string;
}

export type ImageGalleryProps = {
  images: GalleryImage[];
  initialIndex?: number;
  onIndexChange?: (index: number) => void;
  showThumbnails?: boolean;
  ariaLabel?: string;
}
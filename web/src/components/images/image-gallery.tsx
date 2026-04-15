"use client";

import { ImageCard } from "@/components/images/image-card";
import type { GeneratedImage } from "@/lib/user-api";

interface ImageGalleryProps {
  images: GeneratedImage[];
  emptyTitle?: string;
  emptyDescription?: string;
}

export function ImageGallery({
  images,
  emptyTitle = "No images yet",
  emptyDescription = "Generate a prompt to start building your visual library.",
}: ImageGalleryProps) {
  if (!images.length) {
    return (
      <div className="rounded-3xl border border-dashed border-white/10 bg-white/5 p-8 text-center">
        <p className="text-sm font-medium text-white">{emptyTitle}</p>
        <p className="mt-2 text-sm text-slate-400">{emptyDescription}</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
      {images.map((image) => (
        <ImageCard key={image.id} image={image} />
      ))}
    </div>
  );
}

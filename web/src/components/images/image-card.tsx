"use client";

import { useState } from "react";
import { Copy, Download, Expand } from "lucide-react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { GeneratedImage } from "@/lib/user-api";

interface ImageCardProps {
  image: GeneratedImage;
}

export function ImageCard({ image }: ImageCardProps) {
  const [open, setOpen] = useState(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(image.url);
  }

  return (
    <>
      <div className="group overflow-hidden rounded-3xl border border-white/10 bg-[#131314]">
        <div className="relative aspect-square overflow-hidden bg-black">
          <img
            src={image.url}
            alt={image.id}
            className="h-full w-full object-cover transition duration-300 group-hover:scale-[1.03]"
          />
          <div className="absolute inset-0 flex items-end justify-between bg-gradient-to-t from-black/70 via-black/10 to-transparent p-3 opacity-0 transition group-hover:opacity-100">
            <Badge variant="outline" className="border-white/20 bg-black/35 text-white">
              {image.id.slice(0, 10)}
            </Badge>
            <div className="flex items-center gap-2">
              <Button size="icon-sm" variant="secondary" onClick={() => setOpen(true)}>
                <Expand className="h-4 w-4" />
              </Button>
              <Button size="icon-sm" variant="secondary" onClick={() => void handleCopy()}>
                <Copy className="h-4 w-4" />
              </Button>
              <Button size="icon-sm" variant="secondary" onClick={() => window.open(image.url, "_blank")}>
                <Download className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-[min(92vw,72rem)] overflow-hidden border-white/10 bg-[#0e0e0f] p-0 text-white">
          <DialogHeader className="px-4 pt-4">
            <DialogTitle className="text-sm text-slate-200">{image.id}</DialogTitle>
          </DialogHeader>
          <div className="max-h-[80vh] overflow-auto p-4 pt-0">
            <img src={image.url} alt={image.id} className="w-full rounded-2xl object-contain" />
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}

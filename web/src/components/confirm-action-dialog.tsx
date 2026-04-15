'use client';

import { Button } from '@/components/ui/button';
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog';

interface ConfirmActionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  confirmLabel?: string;
  cancelLabel?: string;
  loading?: boolean;
  theme?: 'default' | 'chat';
  onConfirm: () => void | Promise<void>;
}

export function ConfirmActionDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  loading = false,
  theme = 'default',
  onConfirm,
}: ConfirmActionDialogProps) {
  const isChatTheme = theme === 'chat';

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        showCloseButton={false}
        className={isChatTheme ? "max-w-md border-white/10 bg-[#171717] text-[#e2e2e2] shadow-[0_24px_80px_rgba(0,0,0,0.5)]" : "max-w-md"}
      >
        <DialogHeader>
          <DialogTitle className={isChatTheme ? "text-white" : undefined}>{title}</DialogTitle>
          <DialogDescription className={isChatTheme ? "text-[#b8b8b8]" : undefined}>
            {description}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className={isChatTheme ? "!mx-0 !mb-0 border-white/10 bg-transparent px-0 pt-2" : "!mx-0 !mb-0 px-0 pt-2"}>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={loading}
            className={isChatTheme ? "border-white/10 bg-transparent text-[#d4d4d4] hover:bg-white/6 hover:text-white" : undefined}
          >
            {cancelLabel}
          </Button>
          <Button
            variant={isChatTheme ? "default" : "destructive"}
            onClick={() => void onConfirm()}
            disabled={loading}
            className={isChatTheme ? "bg-white text-[#1a1c1c] hover:bg-[#d6d6d6]" : undefined}
          >
            {loading ? 'Processing...' : confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

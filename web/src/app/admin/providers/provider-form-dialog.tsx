'use client';

import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { createProvider, updateProvider, type ProviderSummary } from '@/lib/api';

interface ProviderFormDialogProps {
  open: boolean;
  provider: ProviderSummary | null;
  onOpenChange: (open: boolean) => void;
  onSaved: () => Promise<void> | void;
}

export function ProviderFormDialog({
  open,
  provider,
  onOpenChange,
  onSaved,
}: ProviderFormDialogProps) {
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setName(provider?.name || '');
    setSlug(provider?.slug || '');
    setError(null);
  }, [open, provider]);

  const handleSave = async () => {
    const trimmedName = name.trim();
    const normalizedSlug = slug.trim().toLowerCase();

    if (!trimmedName) {
      setError('Provider name is required.');
      return;
    }
    if (!normalizedSlug || !/^[a-z0-9_-]+$/.test(normalizedSlug)) {
      setError('Slug must contain only lowercase letters, numbers, underscores, or hyphens.');
      return;
    }

    setSaving(true);
    setError(null);
    try {
      if (provider) {
        await updateProvider(provider.id, { name: trimmedName });
      } else {
        await createProvider({ name: trimmedName, slug: normalizedSlug });
      }
      await onSaved();
      onOpenChange(false);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : 'Failed to save provider.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{provider ? 'Edit Provider' : 'Create Provider'}</DialogTitle>
          <DialogDescription>
            Provider là cấp gốc để nhóm account pool và model catalog theo từng upstream.
          </DialogDescription>
        </DialogHeader>

        {error && <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{error}</div>}

        <div className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Name</label>
            <Input value={name} onChange={(event) => setName(event.target.value)} placeholder="Codex" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Slug</label>
            <Input
              value={slug}
              onChange={(event) => setSlug(event.target.value)}
              placeholder="codex"
              disabled={Boolean(provider)}
            />
            <p className="text-xs text-slate-500">Slug khóa identity runtime. Tạo xong thì không đổi ở màn này.</p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving ? 'Saving...' : provider ? 'Save Changes' : 'Create Provider'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

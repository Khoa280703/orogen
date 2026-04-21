'use client';

import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  createProviderModel,
  updateProviderModel,
  type ProviderModelSummary,
  type ProviderSummary,
} from '@/lib/api';

interface ModelFormDialogProps {
  open: boolean;
  model: ProviderModelSummary | null;
  providers: ProviderSummary[];
  lockedProvider?: ProviderSummary | null;
  onOpenChange: (open: boolean) => void;
  onSaved: () => Promise<void> | void;
}

export function ModelFormDialog({
  open,
  model,
  providers,
  lockedProvider = null,
  onOpenChange,
  onSaved,
}: ModelFormDialogProps) {
  const [providerId, setProviderId] = useState('none');
  const [name, setName] = useState('');
  const [slug, setSlug] = useState('');
  const [description, setDescription] = useState('');
  const [sortOrder, setSortOrder] = useState('0');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setProviderId(model ? String(model.provider_id) : lockedProvider ? String(lockedProvider.id) : 'none');
    setName(model?.name || '');
    setSlug(model?.slug || '');
    setDescription(model?.description || '');
    setSortOrder(String(model?.sort_order ?? 0));
    setError(null);
  }, [open, model, lockedProvider]);

  const handleSave = async () => {
    const trimmedName = name.trim();
    const normalizedSlug = slug.trim();
    const parsedProviderId = Number(providerId);
    const parsedSortOrder = Number(sortOrder || '0');

    if (!Number.isInteger(parsedProviderId) || parsedProviderId <= 0) {
      setError('Select a provider first.');
      return;
    }
    if (!trimmedName) {
      setError('Model name is required.');
      return;
    }
    if (!normalizedSlug) {
      setError('Model slug is required.');
      return;
    }

    setSaving(true);
    setError(null);
    try {
      if (model) {
        await updateProviderModel(model.id, {
          name: trimmedName,
          description: description.trim() || undefined,
          sort_order: Number.isFinite(parsedSortOrder) ? parsedSortOrder : 0,
        });
      } else {
        await createProviderModel({
          provider_id: parsedProviderId,
          name: trimmedName,
          slug: normalizedSlug,
          description: description.trim() || undefined,
          sort_order: Number.isFinite(parsedSortOrder) ? parsedSortOrder : 0,
        });
      }
      await onSaved();
      onOpenChange(false);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : 'Failed to save model.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{model ? 'Edit Model' : 'Create Model'}</DialogTitle>
          <DialogDescription>
            Catalog model xác định slug public, provider gốc và thứ tự hiển thị trong hệ thống.
          </DialogDescription>
        </DialogHeader>

        {error && <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{error}</div>}

        <div className="space-y-4">
          {lockedProvider ? (
            <div className="space-y-2">
              <label className="text-sm font-medium">Provider</label>
              <div className="rounded-md border bg-slate-50 px-3 py-2 text-sm text-slate-700">
                {lockedProvider.name} ({lockedProvider.slug})
              </div>
            </div>
          ) : (
            <div className="space-y-2">
              <label className="text-sm font-medium">Provider</label>
              <Select value={providerId} onValueChange={(value) => setProviderId(value || 'none')} disabled={Boolean(model)}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select provider" />
                </SelectTrigger>
                <SelectContent>
                  {providers.map((provider) => (
                    <SelectItem key={provider.id} value={String(provider.id)}>
                      {provider.name} ({provider.slug})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
          <div className="space-y-2">
            <label className="text-sm font-medium">Name</label>
            <Input value={name} onChange={(event) => setName(event.target.value)} placeholder="GPT-5.1" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Slug</label>
            <Input
              value={slug}
              onChange={(event) => setSlug(event.target.value)}
              placeholder="gpt-5.1"
              disabled={Boolean(model)}
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Sort Order</label>
            <Input value={sortOrder} onChange={(event) => setSortOrder(event.target.value)} type="number" />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">Description</label>
            <Textarea
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Optional description for admin context"
              rows={4}
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={saving}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving ? 'Saving...' : model ? 'Save Changes' : 'Create Model'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

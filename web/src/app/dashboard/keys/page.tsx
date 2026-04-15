"use client";

export const dynamic = "force-dynamic";

import { useState, useEffect } from "react";
import { Key, Plus, Copy, Trash2 } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ConfirmActionDialog } from "@/components/confirm-action-dialog";
import { Input } from "@/components/ui/input";
import { userApiFetch, userApiRequest } from "@/lib/user-api";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface ApiKey {
  id: number;
  label: string;
  key_prefix: string;
  created_at: string;
  last_used_at?: string;
  active: boolean;
}

export default function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [showDialog, setShowDialog] = useState(false);
  const [newKeyLabel, setNewKeyLabel] = useState("");
  const [newKey, setNewKey] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [revokeTarget, setRevokeTarget] = useState<ApiKey | null>(null);
  const [revoking, setRevoking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchKeys = async () => {
    try {
      setError(null);
      const data = await userApiRequest<{ keys?: ApiKey[] }>("/user/keys");
      setKeys(data.keys || []);
    } catch (error) {
      setError(error instanceof Error ? error.message : "Failed to load API keys.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void fetchKeys();
  }, []);

  const handleCreateKey = async () => {
    if (!newKeyLabel.trim()) return;

    setCreating(true);
    try {
      setError(null);
      const data = await userApiRequest<{ key: string }>("/user/keys", {
        method: "POST",
        body: JSON.stringify({ label: newKeyLabel }),
      });
      setNewKey(data.key);
      await fetchKeys();
    } catch (error) {
      setError(error instanceof Error ? error.message : "Failed to create API key.");
    } finally {
      setCreating(false);
    }
  };

  const handleRevokeKey = async () => {
    if (!revokeTarget) return;

    setRevoking(true);
    try {
      setError(null);
      await userApiFetch(`/user/keys/${revokeTarget.id}`, {
        method: "DELETE",
      });
      setRevokeTarget(null);
      await fetchKeys();
    } catch (error) {
      setError(error instanceof Error ? error.message : "Failed to revoke API key.");
    } finally {
      setRevoking(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold">API Keys</h1>
          <p className="text-slate-400 mt-1">Manage your API keys for programmatic access</p>
        </div>
        <Button onClick={() => setShowDialog(true)}>
          <Plus className="w-5 h-5 mr-2" />
          Create New Key
        </Button>
      </div>

      <Card className="bg-slate-900 border-slate-800">
        <CardHeader>
          <CardTitle>Your API Keys</CardTitle>
          <CardDescription>
            Keep your keys secure. Never share them publicly or commit them to version control.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error && (
            <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
              {error}
            </div>
          )}
          {loading ? (
            <div className="flex justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
            </div>
          ) : keys.length === 0 ? (
            <div className="text-center py-8 text-slate-400">
              <Key className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p>No API keys yet. Create one to get started.</p>
            </div>
          ) : (
            <div className="space-y-4">
              {keys.map((key) => (
                <div
                  key={key.id}
                  className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg border border-slate-700"
                >
                  <div className="flex-1">
                    <div className="font-mono text-lg">{key.key_prefix}...</div>
                    <div className="text-sm text-slate-400 mt-1">
                      {key.label} • Created {new Date(key.created_at).toLocaleDateString()}
                      {key.last_used_at && ` • Last used ${new Date(key.last_used_at).toLocaleDateString()}`}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button variant="ghost" size="icon" onClick={() => copyToClipboard(key.key_prefix)}>
                      <Copy className="w-5 h-5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => setRevokeTarget(key)}
                      className="text-red-400 hover:text-red-300 hover:bg-red-500/10"
                    >
                      <Trash2 className="w-5 h-5" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Create Key Dialog */}
      <Dialog open={showDialog} onOpenChange={setShowDialog}>
        <DialogContent className="bg-slate-900 border-slate-800">
          <DialogHeader>
            <DialogTitle>Create New API Key</DialogTitle>
            <DialogDescription>
              Give your key a descriptive label to identify it later.
            </DialogDescription>
          </DialogHeader>

          {newKey ? (
            <div className="space-y-4 py-4">
              <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <p className="text-green-400 text-sm mb-2">Key created successfully!</p>
                <p className="font-mono text-sm break-all">{newKey}</p>
                <p className="text-red-400 text-xs mt-2">
                  ⚠️ This is the only time you&apos;ll see this key. Save it securely!
                </p>
              </div>
              <Button onClick={() => copyToClipboard(newKey)} variant="outline">
                <Copy className="w-4 h-4 mr-2" />
                Copy to Clipboard
              </Button>
            </div>
          ) : (
            <div className="space-y-4 py-4">
              <div>
                <label className="text-sm font-medium mb-2 block">Label</label>
                <Input
                  value={newKeyLabel}
                  onChange={(e) => setNewKeyLabel(e.target.value)}
                  placeholder="e.g., Production server, Mobile app"
                />
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setShowDialog(false)}>
                  Cancel
                </Button>
                <Button onClick={handleCreateKey} disabled={!newKeyLabel.trim() || creating}>
                  {creating ? "Creating..." : "Create Key"}
                </Button>
              </DialogFooter>
            </div>
          )}
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        open={!!revokeTarget}
        onOpenChange={(open) => !open && setRevokeTarget(null)}
        title="Revoke API key?"
        description={`Key ${revokeTarget?.label || revokeTarget?.key_prefix || ''} will stop working immediately.`}
        confirmLabel="Revoke Key"
        loading={revoking}
        onConfirm={handleRevokeKey}
      />
    </div>
  );
}

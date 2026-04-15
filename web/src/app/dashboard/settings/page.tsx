"use client";

export const dynamic = "force-dynamic";

import { useState, useEffect } from "react";
import Link from "next/link";
import { User, Globe, ArrowUpRight } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { userApiRequest } from "@/lib/user-api";

interface UserProfile {
  user: {
    id: number;
    email: string;
    name?: string;
    avatar_url?: string;
    locale: string;
  };
  plan?: {
    name: string;
    slug: string;
    requests_per_day?: number;
    price_usd?: string;
  };
}

export default function SettingsPage() {
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [name, setName] = useState("");
  const [locale, setLocale] = useState("en");
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<{ type: "success" | "error"; message: string } | null>(null);

  useEffect(() => {
    const fetchProfile = async () => {
      try {
        setNotice(null);
        const data = await userApiRequest<UserProfile>("/user/me");
        setProfile(data);
        setName(data.user.name || "");
        setLocale(data.user.locale || "en");
      } catch (error) {
        setNotice({
          type: "error",
          message: error instanceof Error ? error.message : "Failed to load profile.",
        });
      } finally {
        setLoading(false);
      }
    };

    fetchProfile();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    try {
      setNotice(null);
      await userApiRequest("/user/me", {
        method: "PUT",
        body: JSON.stringify({ name, locale }),
      });
      setNotice({ type: "success", message: "Settings saved successfully." });
    } catch (error) {
      setNotice({
        type: "error",
        message: error instanceof Error ? error.message : "Failed to save settings.",
      });
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Settings</h1>
        <p className="text-slate-400 mt-1">Manage your account preferences</p>
      </div>

      {notice && (
        <div
          className={`rounded-lg px-4 py-3 text-sm ${
            notice.type === "success"
              ? "border border-green-500/30 bg-green-500/10 text-green-200"
              : "border border-red-500/30 bg-red-500/10 text-red-200"
          }`}
        >
          {notice.message}
        </div>
      )}

      {/* Profile Settings */}
      <Card className="bg-slate-900 border-slate-800">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <User className="w-5 h-5" />
            Profile Information
          </CardTitle>
          <CardDescription>Update your personal details</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center gap-4">
            {profile?.user.avatar_url ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={profile.user.avatar_url}
                alt={profile.user.name || "Avatar"}
                className="h-20 w-20 rounded-full border-2 border-slate-700 object-cover"
              />
            ) : (
              <div className="w-20 h-20 rounded-full bg-slate-700 flex items-center justify-center">
                <span className="text-2xl font-bold text-slate-400">
                  {(profile?.user.name || profile?.user.email || "?")[0].toUpperCase()}
                </span>
              </div>
            )}
            <div>
              <div className="font-medium">{profile?.user.name || "No name set"}</div>
              <div className="text-sm text-slate-400">{profile?.user.email}</div>
            </div>
          </div>

          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium mb-2 block">Display Name</label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Enter your display name"
              />
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Email</label>
              <Input
                value={profile?.user.email || ""}
                disabled
                className="bg-slate-800 text-slate-400"
              />
              <p className="text-xs text-slate-500 mt-1">
                Email cannot be changed. Contact support if you need assistance.
              </p>
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">
                <Globe className="w-4 h-4 inline mr-1" />
                Language
              </label>
              <select
                value={locale}
                onChange={(e) => setLocale(e.target.value)}
                className="w-full bg-slate-800 border border-slate-700 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="en">English</option>
                <option value="vi">Vietnamese</option>
                <option value="zh">中文</option>
                <option value="ja">日本語</option>
              </select>
            </div>
          </div>

          <div className="flex justify-end">
            <Button onClick={handleSave} disabled={saving}>
              {saving ? "Saving..." : "Save Changes"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Current Plan */}
      <Card className="bg-slate-900 border-slate-800">
        <CardHeader>
          <CardTitle>Current Plan</CardTitle>
          <CardDescription>Your current subscription details</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <div className="font-medium text-lg">{profile?.plan?.name || "Free"}</div>
              <div className="text-sm text-slate-400">
                {profile?.plan?.requests_per_day === -1
                  ? "Unlimited requests"
                  : `${profile?.plan?.requests_per_day || 0} requests/day`}
              </div>
            </div>
            <Link href="/pricing">
              <Button variant="outline" className="w-full">
                <ArrowUpRight className="w-5 h-5 mr-2" />
                Upgrade
              </Button>
            </Link>
          </div>
        </CardContent>
      </Card>

      {/* API Access */}
      <Card className="bg-slate-900 border-slate-800">
        <CardHeader>
          <CardTitle>API Access</CardTitle>
          <CardDescription>Manage your API access and integrations</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <div className="font-medium">API Keys</div>
              <div className="text-sm text-slate-400">Create and manage your API keys</div>
            </div>
            <Link href="/dashboard/keys">
              <Button variant="ghost">Manage</Button>
            </Link>
          </div>

          <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
            <div>
              <div className="font-medium">Documentation</div>
              <div className="text-sm text-slate-400">Read the API documentation</div>
            </div>
            <Link href="/docs" target="_blank">
              <Button variant="ghost">View Docs</Button>
            </Link>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

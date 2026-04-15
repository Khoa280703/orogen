"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LogIn } from "lucide-react";
import { buildApiUrl } from "@/lib/api-base-url";

export default function LoginPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(() =>
    typeof window === "undefined"
      ? null
      : new URLSearchParams(window.location.search).get("error")
  );

  useEffect(() => {
    const checkSession = async () => {
      try {
        const response = await fetch(buildApiUrl("/user/me"), {
          credentials: "include",
        });
        if (response.ok) {
          router.replace("/chat?new=1");
        }
      } catch {
        // ignore unauthenticated state on login page
      }
    };

    checkSession();
  }, [router]);

  async function handleGoogleLogin() {
    setLoading(true);
    setError(null);

    try {
      window.location.href = "/api/auth/google";
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Login failed");
      setLoading(false);
    }
  }

  return (
    <div className="min-h-[80vh] flex items-center justify-center px-4 py-12">
        <Card className="w-full max-w-md bg-slate-950 border-slate-800">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">Welcome to Media Studio</CardTitle>
          <p className="text-slate-400 mt-2">
            Sign in to access your studio workspace, usage, and API keys
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
              {error}
            </div>
          )}

          <Button
            onClick={handleGoogleLogin}
            disabled={loading}
            className="w-full h-12"
            variant="outline"
          >
            <LogIn className="w-5 h-5 mr-2" />
            {loading ? "Signing in..." : "Continue with Google"}
          </Button>

          <div className="text-center text-sm text-slate-500">
            By continuing, you agree to our{" "}
            <a href="/terms" className="underline hover:text-white">
              Terms of Service
            </a>{" "}
            and{" "}
            <a href="/privacy" className="underline hover:text-white">
              Privacy Policy
            </a>
          </div>

          <div className="text-center text-sm text-slate-500">
            Already have an account?{" "}
            <a href="/chat" className="underline hover:text-white">
              Go to chat
            </a>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

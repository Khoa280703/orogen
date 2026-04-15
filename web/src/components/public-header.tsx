"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Globe } from "lucide-react";

export function PublicHeader() {
  return (
    <header className="border-b border-slate-800 bg-slate-950/80 backdrop-blur sticky top-0 z-50">
      <div className="container mx-auto px-4 h-16 flex items-center justify-between">
        <Link href="/" className="flex items-center gap-2">
          <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg" />
          <span className="font-bold text-xl">Grok API</span>
        </Link>

        <nav className="flex items-center gap-6">
          <Link href="/#features" className="text-slate-400 hover:text-white transition">
            Features
          </Link>
          <Link href="/pricing" className="text-slate-400 hover:text-white transition">
            Pricing
          </Link>
          <Link href="/docs" className="text-slate-400 hover:text-white transition">
            Docs
          </Link>
          <div className="w-px h-4 bg-slate-700" />
          <Button variant="ghost" size="sm" className="text-slate-400">
            <Globe className="w-4 h-4 mr-2" />
            EN
          </Button>
          <Link href="/login">
            <Button size="sm">Login</Button>
          </Link>
        </nav>
      </div>
    </header>
  );
}

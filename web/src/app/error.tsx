'use client';

import { Button } from '@/components/ui/button';

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-slate-950 text-white">
      <div className="text-center space-y-6">
        <h1 className="text-6xl font-bold text-red-500">500</h1>
        <h2 className="text-2xl font-semibold">Something Went Wrong</h2>
        <p className="text-slate-400 max-w-md">
          {error.message || 'An unexpected error occurred. Please try again later.'}
        </p>
        <div className="flex gap-4 justify-center">
          <Button onClick={reset}>Try Again</Button>
          <Button variant="outline" onClick={() => (window.location.href = '/')}>
            Go Home
          </Button>
        </div>
      </div>
    </div>
  );
}

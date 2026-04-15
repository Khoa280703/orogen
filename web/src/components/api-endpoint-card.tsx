'use client';

import { Badge } from '@/components/ui/badge';

interface EndpointCardProps {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  path: string;
  description: string;
}

const methodColors: Record<string, string> = {
  GET: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300',
  POST: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
  PUT: 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300',
  DELETE: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300',
};

export function ApiEndpointCard({ method, path, description }: EndpointCardProps) {
  return (
    <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4 space-y-2">
      <div className="flex items-center gap-3">
        <Badge className={methodColors[method]}>
          {method}
        </Badge>
        <code className="text-sm font-mono">{path}</code>
      </div>
      <p className="text-sm text-slate-600 dark:text-slate-400">
        {description}
      </p>
    </div>
  );
}

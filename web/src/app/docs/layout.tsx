import { Metadata } from 'next';
import { DocsSidebar } from '@/components/docs-sidebar';
import { PublicSiteShell } from '@/components/public-site-shell';

export const metadata: Metadata = {
  title: 'Documentation - Grok API',
  description: 'Complete API documentation and guides for building AI applications with Grok.',
};

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <PublicSiteShell contentClassName="min-h-screen bg-white text-slate-950">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          <aside className="hidden lg:block lg:col-span-1">
            <div className="sticky top-8">
              <h2 className="text-lg font-semibold mb-4">Documentation</h2>
              <DocsSidebar />
            </div>
          </aside>
          <main className="lg:col-span-3">{children}</main>
        </div>
      </div>
    </PublicSiteShell>
  );
}

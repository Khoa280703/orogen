import { Metadata } from 'next';
import { PublicSiteShell } from '@/components/public-site-shell';

export const metadata: Metadata = {
  title: 'Pricing - Grok API',
  description: 'Choose the perfect plan for your AI needs. From free tier to enterprise solutions.',
  openGraph: {
    title: 'Pricing - Grok API',
    description: 'Choose the perfect plan for your AI needs.',
    type: 'website',
  },
};

export default function PricingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <PublicSiteShell>{children}</PublicSiteShell>;
}

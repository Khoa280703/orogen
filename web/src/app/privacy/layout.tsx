import { PublicSiteShell } from "@/components/public-site-shell";

export default function PrivacyLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <PublicSiteShell>{children}</PublicSiteShell>;
}

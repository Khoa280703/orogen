import { PublicFooter } from "@/components/public-footer";
import { PublicHeader } from "@/components/public-header";

export function PublicSiteShell({
  children,
  contentClassName = "min-h-screen",
}: {
  children: React.ReactNode;
  contentClassName?: string;
}) {
  return (
    <div className="min-h-screen bg-slate-950 text-white">
      <PublicHeader />
      <main className={contentClassName}>{children}</main>
      <PublicFooter />
    </div>
  );
}

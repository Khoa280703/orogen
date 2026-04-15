import { Inter } from "next/font/google";
import { AppLayoutShell } from "@/components/app-layout-shell";

const inter = Inter({ subsets: ["latin"] });

export const dynamic = "force-dynamic";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <AppLayoutShell className={inter.className}>{children}</AppLayoutShell>;
}

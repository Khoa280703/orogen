import { Manrope } from "next/font/google";
import { ChatShell } from "@/components/chat/chat-shell";

const manrope = Manrope({
  subsets: ["latin"],
  variable: "--font-chat-headline",
});

export const dynamic = "force-dynamic";

export default function VideosLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <ChatShell className={manrope.variable}>{children}</ChatShell>;
}

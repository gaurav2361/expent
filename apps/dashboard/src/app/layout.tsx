import type { Metadata } from "next";
import "@expent/ui/globals.css";
import { Providers } from "@/components/providers";

export const metadata: Metadata = {
  title: "Expent",
  description: "Manage your expenses intelligently.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="min-h-screen bg-background font-sans antialiased">
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}

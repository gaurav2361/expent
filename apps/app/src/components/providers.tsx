"use client";

import { Toaster } from "@expent/ui/components/goey-toaster";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider, useTheme } from "@/components/theme-provider";
import { MotionConfig } from "motion/react";
import { useState } from "react";

function AppToaster() {
  const { theme } = useTheme();
  const isDark =
    theme === "dark" ||
    (theme === "system" && typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  return <Toaster theme={isDark ? "dark" : "light"} position="bottom-right" closeButton />;
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 1000 * 60 * 5, // 5 minutes
      },
    },
  }));

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider defaultTheme="dark" storageKey="expent-next-theme">
        <MotionConfig reducedMotion="user">
          {children}
          <AppToaster />
        </MotionConfig>
      </ThemeProvider>
    </QueryClientProvider>
  );
}

import appCss from "@expent/ui/globals.css?url";
import { QueryClientProvider } from "@tanstack/react-query";
import { createRootRoute, HeadContent, Scripts } from "@tanstack/react-router";
import { MotionConfig } from "motion/react";
import { NotFoundPage } from "@/components/not-found";
import { ThemeProvider } from "@/components/theme-provider";
import { queryClient } from "@/lib/query-client";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: "utf-8",
      },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      {
        title: "TanStack Start Starter",
      },
    ],
    links: [
      {
        rel: "stylesheet",
        href: appCss,
      },
    ],
  }),
  shellComponent: RootDocument,
  notFoundComponent: NotFoundPage,
});

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body>
        <QueryClientProvider client={queryClient}>
          <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
            <MotionConfig reducedMotion="user">{children}</MotionConfig>
          </ThemeProvider>
        </QueryClientProvider>
        <Scripts />
      </body>
    </html>
  );
}

"use client";

import type React from "react";
import { FloatingPaths } from "@/components/floating-paths";
import { Logo } from "@/components/logo";

interface AuthPage {
  children: React.ReactNode;
}

export function AuthPage({ children }: AuthPage) {
  return (
    <main className="relative md:h-screen md:overflow-hidden lg:grid lg:grid-cols-2">
      {/* Left Side: Branding and Quote */}
      <div className="relative hidden h-full flex-col border-r bg-secondary p-10 lg:flex dark:bg-secondary/20">
        <div className="absolute inset-0 bg-linear-to-b from-transparent via-transparent to-background" />
        <Logo className="mr-auto h-4.5" />

        <div className="z-10 mt-auto">
          <blockquote className="space-y-2">
            <p className="text-xl">
              &ldquo;This Platform has helped me to save time and serve my clients faster than ever before.&rdquo;
            </p>
            <footer className="font-mono font-semibold text-sm">~ Ali Hassan</footer>
          </blockquote>
        </div>
        <div className="absolute inset-0">
          <FloatingPaths position={1} />
          <FloatingPaths position={-1} />
        </div>
      </div>
      {children}
    </main>
  );
}

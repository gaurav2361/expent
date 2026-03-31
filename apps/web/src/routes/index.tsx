import { Button } from "@expent/ui/components/button";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { Logo } from "@/components/logo";
import { useSession } from "@/lib/auth";

export const Route = createFileRoute("/")({ component: App });

function App() {
  const navigate = useNavigate();
  const session = useSession();

  useEffect(() => {
    if (session.data) {
      navigate({ to: "/dashboard" });
    }
  }, [session.data, navigate]);

  return (
    <div className="flex flex-col min-h-svh">
      <header className="flex h-16 items-center px-8 border-b justify-between">
        <Logo className="h-6" />
        <div className="flex gap-4">
          <Button variant="ghost" render={<Link to="/sign-in" />} nativeButton={false}>
            Sign In
          </Button>
          <Button render={<Link to="/sign-up" />} nativeButton={false}>
            Get Started
          </Button>
        </div>
      </header>

      <main className="flex-1 flex flex-col items-center justify-center text-center px-6 py-24 space-y-8 max-w-4xl mx-auto">
        <h1 className="text-5xl md:text-7xl font-bold tracking-tight">
          Manage your expenses <span className="text-primary">intelligently.</span>
        </h1>
        <p className="text-xl text-muted-foreground max-w-2xl">
          Expent uses OCR and Smart Merge technology to track your spending, deduplicate transactions, and sync shared
          payments with friends automatically.
        </p>
        <div className="flex gap-4">
          <Button size="lg" className="h-12 px-8 text-lg" render={<Link to="/sign-up" />} nativeButton={false}>
            Start tracking now
          </Button>
          <Button size="lg" variant="outline" className="h-12 px-8 text-lg">
            View Demo
          </Button>
        </div>
      </main>

      <footer className="h-16 border-t flex items-center justify-center text-sm text-muted-foreground">
        &copy; 2026 Expent. Built with Rust and TanStack.
      </footer>
    </div>
  );
}

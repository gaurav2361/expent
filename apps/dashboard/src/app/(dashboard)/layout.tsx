"use client";

import { useSession } from "@/lib/auth-client";
import { useRouter } from "next/navigation";
import { useEffect } from "react";
import { SidebarWrapper } from "@/components/sidebar-wrapper";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const session = useSession();
  const router = useRouter();

  useEffect(() => {
    if (!session.isPending && !session.data) {
      router.push("/sign-in");
    }
  }, [session.data, session.isPending, router]);

  if (session.isPending) {
    return <div className="flex h-screen items-center justify-center">Loading...</div>;
  }

  if (!session.data) {
    return null;
  }

  return <SidebarWrapper>{children}</SidebarWrapper>;
}

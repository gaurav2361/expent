"use client";

import { SidebarProvider, SidebarInset } from "@expent/ui/components/sidebar";
import { AppSidebar } from "@/components/layout/app-sidebar";
import { AppNavbar } from "@/components/layout/app-navbar";

export function SidebarClient({ defaultOpen, children }: { defaultOpen: boolean; children: React.ReactNode }) {
  return (
    <SidebarProvider defaultOpen={defaultOpen}>
      <AppSidebar />
      <SidebarInset>
        <AppNavbar />
        <div className="flex flex-1 flex-col">{children}</div>
      </SidebarInset>
    </SidebarProvider>
  );
}

import { SidebarInset, SidebarProvider } from "@expent/ui/components/sidebar";
import { AppNavbar } from "@/components/app-navbar";
import { AppSidebar } from "@/components/app-sidebar";
import { DashboardSkeleton } from "@/components/dashboard-skeleton";

export function AppShell() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <AppNavbar />
        <div className="flex flex-1 flex-col gap-4 p-4 md:p-6">
          <DashboardSkeleton />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

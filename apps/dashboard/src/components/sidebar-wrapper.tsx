import { cookies } from "next/headers";
import { SidebarProvider, SidebarInset } from "@expent/ui/components/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { AppNavbar } from "@/components/app-navbar";

export async function SidebarWrapper({ children }: { children: React.ReactNode }) {
  const cookieStore = await cookies();
  const defaultOpen = cookieStore.get("sidebar_state")?.value === "true";

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

import { cookies } from "next/headers";
import { SidebarClient } from "@/components/sidebar-client";

// Server Component: reads cookie, passes defaultOpen to Client
export async function SidebarWrapper({ children }: { children: React.ReactNode }) {
  const cookieStore = await cookies();
  const sidebarState = cookieStore.get("sidebar_state")?.value;
  // Default to open if no cookie set yet
  const defaultOpen = sidebarState !== "false";

  return (
    <SidebarClient defaultOpen={defaultOpen}>
      {children}
    </SidebarClient>
  );
}

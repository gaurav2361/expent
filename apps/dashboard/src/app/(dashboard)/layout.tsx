import { SidebarWrapper } from "@/components/layout/sidebar-wrapper";
import { AuthGuard } from "@/components/auth/auth-guard";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <AuthGuard>
      <SidebarWrapper>{children}</SidebarWrapper>
    </AuthGuard>
  );
}

import { AuthGuard } from "@/components/auth/auth-guard";
import { SidebarWrapper } from "@/components/layout/sidebar-wrapper";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <AuthGuard>
      <SidebarWrapper>{children}</SidebarWrapper>
    </AuthGuard>
  );
}

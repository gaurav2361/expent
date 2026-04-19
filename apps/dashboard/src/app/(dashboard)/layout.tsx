import { AuthGuard } from "@/components/auth/auth-guard";
import { SidebarWrapper } from "@/components/layout/sidebar-wrapper";
import { CommandCenter } from "@/components/layout/command-center";
import { HotkeyHelp } from "@/components/layout/hotkey-help";
import { GlobalModals } from "@/components/layout/global-modals";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <AuthGuard>
      <SidebarWrapper>
        {children}
        <CommandCenter />
        <HotkeyHelp />
        <GlobalModals />
      </SidebarWrapper>
    </AuthGuard>
  );
}

"use client";

import { Separator } from "@expent/ui/components/separator";
import { SidebarNav } from "@/components/settings/sidebar-nav";
import { UserCogIcon, WrenchIcon, PaletteIcon, BellIcon, MonitorIcon, TagIcon } from "lucide-react";
import { usePathname } from "next/navigation";

const sidebarNavItems = [
  {
    title: "Profile",
    href: "/settings/profile",
    icon: <UserCogIcon className="w-4 h-4" />,
  },
  {
    title: "Account",
    href: "/settings/account",
    icon: <WrenchIcon className="w-4 h-4" />,
  },
  {
    title: "Categories",
    href: "/settings/categories",
    icon: <TagIcon className="w-4 h-4" />,
  },
  {
    title: "Appearance",
    href: "/settings/appearance",
    icon: <PaletteIcon className="w-4 h-4" />,
  },
  {
    title: "Notifications",
    href: "/settings/notifications",
    icon: <BellIcon className="w-4 h-4" />,
  },
  {
    title: "Display",
    href: "/settings/display",
    icon: <MonitorIcon className="w-4 h-4" />,
  },
];

interface SettingsLayoutProps {
  children: React.ReactNode;
}

export default function SettingsLayout({ children }: SettingsLayoutProps) {
  const pathname = usePathname();
  const isIndex = pathname === "/settings";

  return (
    <div className="flex flex-1 flex-col gap-4 p-4 lg:p-8">
      <div className="space-y-0.5">
        <h1 className="text-2xl font-bold tracking-tight md:text-3xl">Settings</h1>
        <p className="text-muted-foreground">Manage your account settings and set e-mail preferences.</p>
      </div>
      <Separator className="my-4" />
      <div className="flex flex-1 flex-col space-y-4 md:space-y-2 lg:flex-row lg:space-y-0 lg:space-x-12">
        {!isIndex && (
          <aside className="top-0 lg:sticky lg:w-1/5 overflow-x-auto pb-2 hidden lg:block">
            <SidebarNav items={sidebarNavItems} />
          </aside>
        )}
        <div className="flex w-full overflow-y-auto p-1">
          {children}
        </div>
      </div>
    </div>
  );
}

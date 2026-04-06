import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
  SidebarRail,
} from "@expent/ui/components/sidebar";
import { Link } from "@tanstack/react-router";
import {
  LayoutDashboardIcon,
  MessageSquareShareIcon,
  ReceiptIcon,
  RepeatIcon,
  Settings2Icon,
  UsersIcon,
} from "lucide-react";
import type * as React from "react";
import { NavMain } from "@/components/nav-main";
import { NavSecondary } from "@/components/nav-secondary";
import { NavUser } from "@/components/nav-user";
import { Logo, LogoIcon } from "./logo";

const data = {
  user: {
    name: "User",
    email: "user@example.com",
    avatar: "",
  },
  navMain: [
    {
      title: "Dashboard",
      url: "/dashboard",
      icon: <LayoutDashboardIcon />,
      isActive: true,
    },
    {
      title: "Transactions",
      url: "/dashboard/transactions",
      icon: <ReceiptIcon />,
    },
    {
      title: "P2P & Sharing",
      url: "/dashboard/p2p",
      icon: <MessageSquareShareIcon />,
      items: [
        {
          title: "Pending Requests",
          url: "/dashboard/p2p/pending",
        },
        {
          title: "Shared Ledgers",
          url: "/dashboard/p2p/shared",
        },
      ],
    },
    {
      title: "Subscriptions",
      url: "/dashboard/subscriptions",
      icon: <RepeatIcon />,
    },
    {
      title: "Contacts",
      url: "/dashboard/contacts",
      icon: <UsersIcon />,
    },
  ],
  navSecondary: [
    {
      title: "Settings",
      url: "/dashboard/settings",
      icon: <Settings2Icon />,
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar variant="inset" collapsible="icon" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <Link
              to="/dashboard"
              className="flex h-8 items-center px-2 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0"
            >
              <Logo className="h-6 w-auto group-data-[collapsible=icon]:hidden" />
              <LogoIcon className="size-6 hidden group-data-[collapsible=icon]:block" />
            </Link>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
        <NavSecondary items={data.navSecondary} className="mt-auto" />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}

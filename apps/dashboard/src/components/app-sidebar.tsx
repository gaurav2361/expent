"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { cn } from "@expent/ui/lib/utils";
import { LogoIcon } from "@/components/logo";
import {
	Sidebar,
	SidebarContent,
	SidebarFooter,
	SidebarGroup,
	SidebarGroupLabel,
	SidebarHeader,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarRail,
} from "@expent/ui/components/sidebar";
import {
	LayoutDashboardIcon,
	ReceiptIcon,
	UsersIcon,
	RepeatIcon,
	Settings2Icon,
	MessageSquareShareIcon,
	ChevronRightIcon,
} from "lucide-react";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "@expent/ui/components/collapsible";
import { SidebarMenuSub, SidebarMenuSubItem, SidebarMenuSubButton, SidebarMenuAction } from "@expent/ui/components/sidebar";

export type SidebarNavItem = {
	title: string;
	url: string;
	icon: React.ReactNode;
	isActive?: boolean;
	items?: { title: string; url: string }[];
};

type SidebarSection = {
	label: string;
	items: SidebarNavItem[];
};

const navSections: SidebarSection[] = [
	{
		label: "Main",
		items: [
			{
				title: "Dashboard",
				url: "/",
				icon: <LayoutDashboardIcon />,
			},
			{
				title: "Transactions",
				url: "/transactions",
				icon: <ReceiptIcon />,
			},
			{
				title: "P2P & Sharing",
				url: "#",
				icon: <MessageSquareShareIcon />,
				items: [
					{
						title: "Pending Requests",
						url: "/p2p/pending",
					},
					{
						title: "Shared Ledgers",
						url: "/p2p/shared-ledgers",
					},
				],
			},
			{
				title: "Subscriptions",
				url: "/subscriptions",
				icon: <RepeatIcon />,
			},
			{
				title: "Contacts",
				url: "/contacts",
				icon: <UsersIcon />,
			},
		],
	},
	{
		label: "Secondary",
		items: [
			{
				title: "Settings",
				url: "/settings",
				icon: <Settings2Icon />,
			},
		],
	},
];


import { NavUser } from "@/components/nav-user";

export function AppSidebar() {
	const pathname = usePathname();

	return (
		<Sidebar
			className={cn(
				"*:data-[slot=sidebar-inner]:bg-background",
				"*:data-[slot=sidebar-inner]:dark:bg-[radial-gradient(60%_18%_at_10%_0%,--theme(--color-foreground/.08),transparent)]",
				"**:data-[slot=sidebar-menu-button]:[&>span]:text-foreground/75"
			)}
			collapsible="icon"
			variant="sidebar"
		>
			<SidebarHeader className="h-14 justify-center border-b px-2">
				<SidebarMenuButton render={<Link href="/" />}><LogoIcon /><span className="font-medium">Efferd</span></SidebarMenuButton>
			</SidebarHeader>
			<SidebarContent>
				{navSections.map((section) => (
					<SidebarGroup key={section.label}>
						<SidebarGroupLabel className="group-data-[collapsible=icon]:pointer-events-none">
							{section.label}
						</SidebarGroupLabel>
						<SidebarMenu>
							{section.items.map((item) => (
								<Collapsible key={item.title} defaultOpen={item.isActive || pathname === item.url || item.items?.some(i => pathname.startsWith(i.url))} render={<SidebarMenuItem />}>
									{item.items?.length ? (
										<CollapsibleTrigger render={<SidebarMenuButton tooltip={item.title} isActive={item.isActive || pathname === item.url || item.items?.some(i => pathname.startsWith(i.url))} />}>
											{item.icon}
											<span>{item.title}</span>
											<ChevronRightIcon className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
										</CollapsibleTrigger>
									) : (
										<SidebarMenuButton isActive={item.isActive || pathname === item.url || (item.url !== "/" && pathname.startsWith(item.url))} tooltip={item.title} render={<Link href={item.url} />}>
											{item.icon}
											<span>{item.title}</span>
										</SidebarMenuButton>
									)}
									{item.items?.length ? (
										<>
											<CollapsibleContent>
												<SidebarMenuSub>
													{item.items?.map((subItem) => (
														<SidebarMenuSubItem key={subItem.title}>
															<SidebarMenuSubButton isActive={pathname === subItem.url || pathname.startsWith(subItem.url)} render={<Link href={subItem.url} />}>
																<span>{subItem.title}</span>
															</SidebarMenuSubButton>
														</SidebarMenuSubItem>
													))}
												</SidebarMenuSub>
											</CollapsibleContent>
										</>
									) : null}
								</Collapsible>
							))}
						</SidebarMenu>
					</SidebarGroup>
				))}
			</SidebarContent>
			<SidebarFooter className="gap-0 p-2">
				<NavUser />
			</SidebarFooter>
			<SidebarRail />
		</Sidebar>
	);
}

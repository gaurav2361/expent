"use client";
import { usePathname } from "next/navigation";
import { cn } from "@expent/ui/lib/utils";
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbList,
	BreadcrumbPage,
	BreadcrumbLink,
	BreadcrumbSeparator,
} from "@expent/ui/components/breadcrumb";
import { Button } from "@expent/ui/components/button";
import { Separator } from "@expent/ui/components/separator";

import { CustomSidebarTrigger } from "@/components/custom-sidebar-trigger";
import { SendIcon, BellIcon } from "lucide-react";

const PATHTITLE: Record<string, string> = {
	"/": "Overview",
	"/transactions": "All Transactions",
	"/p2p/shared-ledgers": "Shared Ledgers",
	"/p2p/pending": "Pending Requests",
	"/p2p/subscriptions": "Subscriptions",
	"/contacts": "Contacts",
	"/settings": "Settings",
};

export function AppNavbar() {
	const pathname = usePathname();
	const title = PATHTITLE[pathname] || "Overview";

	return (
		<header className="flex h-16 shrink-0 items-center justify-between gap-2 px-4 shadow-sm border-b z-10 sticky top-0 bg-background/95 backdrop-blur-sm">
			<div className="flex items-center gap-2">
				<CustomSidebarTrigger />
				<Separator
					className="mr-2 h-4 data-[orientation=vertical]:self-center"
					orientation="vertical"
				/>
				<Breadcrumb>
					<BreadcrumbList>
						<BreadcrumbItem className="hidden md:block">
							<BreadcrumbLink href="/">Dashboard</BreadcrumbLink>
						</BreadcrumbItem>
						{pathname !== "/" && (
							<>
								<BreadcrumbSeparator className="hidden md:block" />
								<BreadcrumbItem>
									<BreadcrumbPage>{title}</BreadcrumbPage>
								</BreadcrumbItem>
							</>
						)}
					</BreadcrumbList>
				</Breadcrumb>
			</div>
			
			<div className="flex items-center gap-3">
				<Button size="icon-sm" variant="outline">
					<SendIcon data-icon="inline-start" />
				</Button>
				<Button aria-label="Notifications" size="icon-sm" variant="outline">
					<BellIcon />
				</Button>
			</div>
		</header>
	);
}

import { createFileRoute } from "@tanstack/react-router";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@expent/ui/components/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { Separator } from "@expent/ui/components/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@expent/ui/components/breadcrumb";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Button } from "@expent/ui/components/button";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@expent/ui/components/dialog";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { PlusIcon, UsersIcon } from "lucide-react";

export const Route = createFileRoute("/dashboard/p2p/shared")({
  component: SharedLedgersComponent,
});

const API_BASE_URL = import.meta.env.VITE_AUTH_BASE_URL || "http://localhost:3001";

function SharedLedgersComponent() {
  const queryClient = useQueryClient();
  const [newGroupName, setNewGroupName] = useState("");
  const [newGroupDesc, setNewGroupDesc] = useState("");
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  // 1. Fetch Groups
  const { data: groups, isLoading } = useQuery({
    queryKey: ["groups"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/groups`, {
          headers: { "Content-Type": "application/json" },
          credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch groups");
      return response.json();
    },
  });

  // 2. Create Group Mutation
  const createMutation = useMutation({
    mutationFn: async () => {
        const response = await fetch(`${API_BASE_URL}/api/groups/create`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name: newGroupName, description: newGroupDesc }),
            credentials: "include",
        });
        if (!response.ok) throw new Error("Failed to create group");
        return response.json();
    },
    onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ["groups"] });
        setIsDialogOpen(false);
        setNewGroupName("");
        setNewGroupDesc("");
    }
  });

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem className="hidden md:block">
                  <BreadcrumbLink href="/dashboard">Dashboard</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <BreadcrumbPage>Shared Ledgers</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
        </header>

        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-bold tracking-tight">Shared Ledgers</h2>
            <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                <DialogTrigger render={<Button />}>
                    <PlusIcon className="mr-2 h-4 w-4" /> New Ledger
                </DialogTrigger>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Create Shared Ledger</DialogTitle>
                        <DialogDescription>
                            Create a space to share and track expenses with friends or family.
                        </DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label htmlFor="name">Ledger Name</Label>
                            <Input 
                                id="name" 
                                value={newGroupName} 
                                onChange={(e) => setNewGroupName(e.target.value)} 
                                placeholder="e.g. Trip to Goa, Apartment Expenses"
                            />
                        </div>
                        <div className="grid gap-2">
                            <Label htmlFor="desc">Description (Optional)</Label>
                            <Input 
                                id="desc" 
                                value={newGroupDesc} 
                                onChange={(e) => setNewGroupDesc(e.target.value)} 
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setIsDialogOpen(false)}>Cancel</Button>
                        <Button onClick={() => createMutation.mutate()} disabled={!newGroupName || createMutation.isPending}>
                            {createMutation.isPending ? "Creating..." : "Create"}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
          </div>

          {isLoading ? (
            <div className="text-center py-20">Loading ledgers...</div>
          ) : groups?.length === 0 ? (
            <Card className="border-dashed">
                <CardContent className="flex flex-col items-center justify-center py-20 text-center">
                    <UsersIcon className="h-12 w-12 text-muted-foreground mb-4" />
                    <h3 className="text-lg font-semibold">No shared ledgers yet</h3>
                    <p className="text-muted-foreground max-w-xs">
                        Create a group to start tracking shared expenses with your contacts.
                    </p>
                </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {groups.map((group: any) => (
                    <Card key={group.id} className="hover:border-primary/50 transition-colors cursor-pointer">
                        <CardHeader>
                            <CardTitle>{group.name}</CardTitle>
                            <CardDescription>{group.description || "No description"}</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="flex items-center text-xs text-muted-foreground">
                                <UsersIcon className="mr-1 h-3 w-3" />
                                Created on {new Date(group.created_at).toLocaleDateString()}
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
          )}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

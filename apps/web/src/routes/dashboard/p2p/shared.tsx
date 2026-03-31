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
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@expent/ui/components/table";
import { Badge } from "@expent/ui/components/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { PlusIcon, UsersIcon, UserPlusIcon, ReceiptIcon, ChevronRightIcon } from "lucide-react";

export const Route = createFileRoute("/dashboard/p2p/shared")({
  component: SharedLedgersComponent,
});

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:3001";

function InviteDialog({ groupId, groupName }: { groupId: string, groupName: string }) {
    const [email, setEmail] = useState("");
    const [open, setOpen] = useState(false);
    const queryClient = useQueryClient();

    const inviteMutation = useMutation({
        mutationFn: async () => {
            const response = await fetch(`${API_BASE_URL}/api/groups/invite`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ group_id: groupId, receiver_email: email }),
                credentials: "include",
            });
            if (!response.ok) throw new Error("Failed to send invite");
            return response.json();
        },
        onSuccess: () => {
            setOpen(false);
            setEmail("");
            alert("Invite sent!");
        }
    });

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger render={<Button size="sm" variant="ghost" className="h-8 w-8 p-0" />}>
                <UserPlusIcon className="h-4 w-4" />
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Invite to {groupName}</DialogTitle>
                    <DialogDescription>
                        Send an invitation to join this shared ledger.
                    </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid gap-2">
                        <Label htmlFor="email">Email Address</Label>
                        <Input 
                            id="email" 
                            type="email"
                            value={email} 
                            onChange={(e) => setEmail(e.target.value)} 
                            placeholder="friend@example.com"
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
                    <Button onClick={() => inviteMutation.mutate()} disabled={!email || inviteMutation.isPending}>
                        {inviteMutation.isPending ? "Sending..." : "Send Invite"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

function GroupDetails({ group }: { group: any }) {
    const { data: transactions, isLoading } = useQuery({
        queryKey: ["group-transactions", group.id],
        queryFn: async () => {
            const response = await fetch(`${API_BASE_URL}/api/groups/${group.id}/transactions`, {
                credentials: "include",
            });
            if (!response.ok) throw new Error("Failed to fetch group transactions");
            return response.json();
        }
    });

    return (
        <div className="space-y-4">
            <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                    <ReceiptIcon className="h-4 w-4" /> Group Activity
                </h3>
            </div>
            
            {isLoading ? (
                <div className="text-center py-10 text-muted-foreground">Loading activity...</div>
            ) : !transactions || transactions.length === 0 ? (
                <div className="text-center py-10 border rounded-lg border-dashed text-muted-foreground">
                    No shared transactions in this ledger yet.
                </div>
            ) : (
                <div className="rounded-md border overflow-hidden">
                    <Table>
                        <TableHeader className="bg-muted/50">
                            <TableRow>
                                <TableHead className="w-[100px]">Date</TableHead>
                                <TableHead>Description</TableHead>
                                <TableHead>Paid By</TableHead>
                                <TableHead className="text-right">Amount</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {transactions.map((txn: any) => (
                                <TableRow key={txn.id}>
                                    <TableCell className="text-xs">
                                        {new Date(txn.date).toLocaleDateString()}
                                    </TableCell>
                                    <TableCell className="font-medium">
                                        {txn.purpose_tag || "Shared Expense"}
                                    </TableCell>
                                    <TableCell>
                                        <Badge variant="outline" className="text-[10px]">
                                            {txn.user_id === "me" ? "You" : "Member"}
                                        </Badge>
                                    </TableCell>
                                    <TableCell className="text-right font-mono font-bold">
                                        ₹{parseFloat(txn.amount).toLocaleString()}
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </div>
            )}
        </div>
    );
}

function SharedLedgersComponent() {
  const queryClient = useQueryClient();
  const [newGroupName, setNewGroupName] = useState("");
  const [newGroupDesc, setNewGroupDesc] = useState("");
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState<any>(null);

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

        <div className="flex flex-1 flex-col gap-6 p-4 pt-0">
          <div className="flex items-center justify-between">
            <div>
                <h2 className="text-2xl font-bold tracking-tight">Shared Ledgers</h2>
                <p className="text-muted-foreground text-sm">Track shared expenses with friends and family.</p>
            </div>
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

          <div className="grid gap-6 lg:grid-cols-3">
            {/* Sidebar List of Groups */}
            <div className="lg:col-span-1 space-y-4">
                {isLoading ? (
                    <div className="text-center py-10">Loading ledgers...</div>
                ) : groups?.length === 0 ? (
                    <Card className="border-dashed">
                        <CardContent className="flex flex-col items-center justify-center py-10 text-center">
                            <UsersIcon className="h-8 w-8 text-muted-foreground mb-2" />
                            <p className="text-sm text-muted-foreground">No ledgers yet.</p>
                        </CardContent>
                    </Card>
                ) : (
                    groups.map((group: any) => (
                        <Card 
                            key={group.id} 
                            className={`hover:border-primary/50 transition-all cursor-pointer group ${selectedGroup?.id === group.id ? 'border-primary ring-1 ring-primary/20' : ''}`}
                            onClick={() => setSelectedGroup(group)}
                        >
                            <CardHeader className="p-4 space-y-0">
                                <div className="flex items-start justify-between">
                                    <div className="space-y-1">
                                        <CardTitle className="text-base">{group.name}</CardTitle>
                                        <CardDescription className="text-xs line-clamp-1">{group.description || "Personal Shared Ledger"}</CardDescription>
                                    </div>
                                    <InviteDialog groupId={group.id} groupName={group.name} />
                                </div>
                            </CardHeader>
                            <CardContent className="px-4 pb-4 pt-0">
                                <div className="flex items-center justify-between mt-2">
                                    <div className="flex items-center text-[10px] text-muted-foreground">
                                        <UsersIcon className="mr-1 h-3 w-3" />
                                        Created {new Date(group.created_at).toLocaleDateString()}
                                    </div>
                                    <ChevronRightIcon className="h-4 w-4 text-muted-foreground group-hover:translate-x-1 transition-transform" />
                                </div>
                            </CardContent>
                        </Card>
                    ))
                )}
            </div>

            {/* Main Content Area: Group Details */}
            <div className="lg:col-span-2">
                {selectedGroup ? (
                    <Card className="min-h-[400px]">
                        <CardHeader className="border-b bg-muted/30">
                            <div className="flex items-center justify-between">
                                <div>
                                    <CardTitle className="text-xl">{selectedGroup.name}</CardTitle>
                                    <CardDescription>{selectedGroup.description || "Shared Ledger Details"}</CardDescription>
                                </div>
                                <Button variant="outline" size="sm">
                                    <UsersIcon className="mr-2 h-4 w-4" /> Manage Members
                                </Button>
                            </div>
                        </CardHeader>
                        <CardContent className="p-6">
                            <GroupDetails group={selectedGroup} />
                        </CardContent>
                    </Card>
                ) : (
                    <Card className="flex flex-col items-center justify-center min-h-[400px] border-dashed">
                        <UsersIcon className="h-12 w-12 text-muted-foreground/30 mb-4" />
                        <h3 className="text-lg font-medium text-muted-foreground">Select a ledger to view details</h3>
                        <p className="text-sm text-muted-foreground/60 max-w-xs text-center mt-2">
                            Choose a shared ledger from the left to see group activity, split status, and manage members.
                        </p>
                    </Card>
                )}
            </div>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

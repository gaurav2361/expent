import { Badge } from "@expent/ui/components/badge";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@expent/ui/components/breadcrumb";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@expent/ui/components/dialog";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Separator } from "@expent/ui/components/separator";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@expent/ui/components/sidebar";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@expent/ui/components/table";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@expent/ui/components/tooltip";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { ChevronRightIcon, InfoIcon, PlusIcon, ReceiptIcon, UserPlusIcon, UsersIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { AppSidebar } from "@/components/app-sidebar";
import { useSession } from "@/lib/auth-client";

export const Route = createFileRoute("/dashboard/p2p/shared")({
  component: SharedLedgersComponent,
});

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:8080";

function InviteDialog({ groupId, groupName }: { groupId: string; groupName: string }) {
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
    },
  });

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger render={<Button size="sm" variant="ghost" className="h-8 w-8 p-0" />}>
        <UserPlusIcon className="h-4 w-4" />
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Invite to {groupName}</DialogTitle>
          <DialogDescription>Send an invitation to join this shared ledger.</DialogDescription>
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
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
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
    },
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <ReceiptIcon className="h-4 w-4 text-primary" /> Recent Activity
        </h3>
        <div className="flex gap-2">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger render={<Button variant="ghost" size="icon" className="h-8 w-8" />}>
                <InfoIcon className="h-4 w-4 text-muted-foreground" />
              </TooltipTrigger>
              <TooltipContent>
                <p>Transactions shared directly with this group</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          <div className="h-12 w-full bg-muted animate-pulse rounded-md" />
          <div className="h-12 w-full bg-muted animate-pulse rounded-md" />
          <div className="h-12 w-full bg-muted animate-pulse rounded-md" />
        </div>
      ) : !transactions || transactions.length === 0 ? (
        <div className="text-center py-16 border rounded-xl border-dashed bg-muted/10">
          <ReceiptIcon className="h-10 w-10 text-muted-foreground/20 mx-auto mb-3" />
          <p className="text-sm text-muted-foreground">No shared transactions yet.</p>
          <p className="text-xs text-muted-foreground/60 mt-1">Upload a receipt and use 'Split' to see it here.</p>
        </div>
      ) : (
        <div className="rounded-xl border overflow-hidden bg-background">
          <Table>
            <TableHeader className="bg-muted/30">
              <TableRow>
                <TableHead className="px-4">Date</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right px-4">Amount</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {transactions.map((txn: any) => (
                <TableRow key={txn.id}>
                  <TableCell className="px-4 text-xs text-muted-foreground">
                    {new Date(txn.date).toLocaleDateString(undefined, { day: "2-digit", month: "short" })}
                  </TableCell>
                  <TableCell className="font-medium">
                    <div className="flex flex-col">
                      <span>{txn.purpose_tag || "Group Expense"}</span>
                      <span className="text-[10px] text-muted-foreground italic">via {txn.source}</span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant="secondary" className="text-[10px] h-5">
                      {txn.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right px-4 font-mono font-bold text-sm">
                    ₹{parseFloat(txn.amount).toLocaleString()}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}

      <div className="pt-4 mt-4 border-t border-dashed">
        <h4 className="text-sm font-semibold mb-3">Itemized Split Status</h4>
        <div className="grid gap-2">
          <div className="p-3 rounded-lg border bg-muted/10 flex items-center justify-between text-sm">
            <div className="flex items-center gap-3">
              <div className="size-8 rounded-full bg-primary/10 flex items-center justify-center font-bold text-xs text-primary">
                CH
              </div>
              <div>
                <p className="font-medium">Shared with 3 people</p>
                <p className="text-[10px] text-muted-foreground">₹450.00 total split value</p>
              </div>
            </div>
            <Badge variant="outline" className="text-[10px] border-orange-200 text-orange-700 bg-orange-50">
              2 Pending
            </Badge>
          </div>
        </div>
      </div>
    </div>
  );
}

function SharedLedgersComponent() {
  const navigate = useNavigate();
  const session = useSession();
  const queryClient = useQueryClient();
  const [newGroupName, setNewGroupName] = useState("");
  const [newGroupDesc, setNewGroupDesc] = useState("");
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState<any>(null);

  useEffect(() => {
    if (!session.isPending && !session.data) {
      navigate({ to: "/sign-in" });
    }
  }, [session.data, session.isPending, navigate]);

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
    enabled: !!session.data,
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
    },
  });

  if (session.isPending) {
    return <div className="flex h-screen items-center justify-center">Loading session...</div>;
  }

  if (!session.data) {
    return null;
  }

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
                    <Input id="desc" value={newGroupDesc} onChange={(e) => setNewGroupDesc(e.target.value)} />
                  </div>
                </div>
                <DialogFooter>
                  <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
                    Cancel
                  </Button>
                  <Button onClick={() => createMutation.mutate()} disabled={!newGroupName || createMutation.isPending}>
                    {createMutation.isPending ? "Creating..." : "Create"}
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>

          <div className="grid gap-6 lg:grid-cols-3">
            <div className="lg:col-span-1 space-y-4">
              {isLoading ? (
                <div className="space-y-3">
                  <div className="h-24 w-full bg-muted animate-pulse rounded-lg" />
                  <div className="h-24 w-full bg-muted animate-pulse rounded-lg" />
                </div>
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
                    className={`hover:border-primary/50 transition-all cursor-pointer group ${selectedGroup?.id === group.id ? "border-primary ring-1 ring-primary/20 shadow-sm" : ""}`}
                    onClick={() => setSelectedGroup(group)}
                  >
                    <CardHeader className="p-4 space-y-0">
                      <div className="flex items-start justify-between">
                        <div className="space-y-1">
                          <CardTitle className="text-base">{group.name}</CardTitle>
                          <CardDescription className="text-xs line-clamp-1 italic">
                            {group.description || "Active Shared Ledger"}
                          </CardDescription>
                        </div>
                        <InviteDialog groupId={group.id} groupName={group.name} />
                      </div>
                    </CardHeader>
                    <CardContent className="px-4 pb-4 pt-0">
                      <div className="flex items-center justify-between mt-2">
                        <div className="flex items-center text-[10px] text-muted-foreground font-medium">
                          <UsersIcon className="mr-1 h-3 w-3" />
                          Created {new Date(group.created_at).toLocaleDateString()}
                        </div>
                        <ChevronRightIcon
                          className={`h-4 w-4 text-muted-foreground transition-transform ${selectedGroup?.id === group.id ? "translate-x-1 text-primary" : "group-hover:translate-x-1"}`}
                        />
                      </div>
                    </CardContent>
                  </Card>
                ))
              )}
            </div>

            <div className="lg:col-span-2">
              {selectedGroup ? (
                <Card className="min-h-[500px] shadow-md border-primary/5">
                  <CardHeader className="border-b bg-muted/10">
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle className="text-xl flex items-center gap-2">
                          {selectedGroup.name}
                          <Badge variant="outline" className="text-[10px] font-normal px-2">
                            ID: {selectedGroup.id.substring(0, 8)}
                          </Badge>
                        </CardTitle>
                        <CardDescription>
                          {selectedGroup.description || "Shared ledger details and activity."}
                        </CardDescription>
                      </div>
                      <Button variant="outline" size="sm" className="shadow-none">
                        <UsersIcon className="mr-2 h-4 w-4" /> Members
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent className="p-6">
                    <GroupDetails group={selectedGroup} />
                  </CardContent>
                </Card>
              ) : (
                <Card className="flex flex-col items-center justify-center min-h-[500px] border-dashed bg-muted/5">
                  <div className="bg-muted p-4 rounded-full mb-4">
                    <UsersIcon className="h-10 w-10 text-muted-foreground/40" />
                  </div>
                  <h3 className="text-lg font-medium text-muted-foreground">Select a Ledger</h3>
                  <p className="text-sm text-muted-foreground/60 max-w-xs text-center mt-2 px-6">
                    Choose a shared ledger from the left sidebar to view group activity, pending splits, and manage your
                    circle.
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

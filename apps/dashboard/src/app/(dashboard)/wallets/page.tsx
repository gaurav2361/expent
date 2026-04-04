"use client";

import * as React from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "@expent/ui/components/card";
import { Button } from "@expent/ui/components/button";
import { Input } from "@expent/ui/components/input";
import { Badge } from "@expent/ui/components/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@expent/ui/components/dialog";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { toast } from "@expent/ui/components/goey-toaster";
import {
  PlusIcon,
  WalletIcon,
  CreditCardIcon,
  BanknoteIcon,
  Building2Icon,
  SmartphoneIcon,
  MoreVerticalIcon,
  PencilIcon,
  Trash2Icon,
} from "lucide-react";

export default function WalletsPage() {
  const queryClient = useQueryClient();
  const [isCreateDialogOpen, setIsCreateDialogOpen] = React.useState(false);
  const [newName, setNewName] = React.useState("");
  const [newType, setNewType] = React.useState("CASH");
  const [newBalance, setNewBalance] = React.useState("0");

  const { data: wallets, isLoading } = useQuery({
    queryKey: ["wallets"],
    queryFn: () => apiClient<any[]>("/api/wallets"),
  });

  const createMutation = useMutation({
    mutationFn: () =>
      apiClient("/api/wallets", {
        method: "POST",
        body: JSON.stringify({
          name: newName,
          type: newType,
          initial_balance: parseFloat(newBalance),
        }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      setIsCreateDialogOpen(false);
      setNewName("");
      setNewType("CASH");
      setNewBalance("0");
      toast.success("Wallet created");
    },
  });

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 md:p-6 lg:p-8 max-w-7xl mx-auto w-full">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Wallets & Accounts</h1>
          <p className="text-muted-foreground text-sm">Manage your payment methods and track balances.</p>
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger render={<Button />}>
            <PlusIcon className="mr-2 h-4 w-4" /> Add Wallet
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create Wallet</DialogTitle>
              <DialogDescription>Add a new bank account, credit card, or cash wallet.</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">Wallet Name</Label>
                <Input
                  id="name"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="e.g. HDFC Bank, My Credit Card"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="type">Type</Label>
                  <Select value={newType} onValueChange={setNewType}>
                    <SelectTrigger id="type">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="CASH">Cash</SelectItem>
                      <SelectItem value="BANK">Bank Account</SelectItem>
                      <SelectItem value="CREDIT_CARD">Credit Card</SelectItem>
                      <SelectItem value="UPI">UPI Wallet</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="balance">Initial Balance (₹)</Label>
                  <Input
                    id="balance"
                    type="number"
                    step="0.01"
                    value={newBalance}
                    onChange={(e) => setNewBalance(e.target.value)}
                  />
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={() => createMutation.mutate()} disabled={!newName || createMutation.isPending}>
                {createMutation.isPending ? "Creating..." : "Create Wallet"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <Card key={i} className="h-40 animate-pulse bg-muted/50" />
          ))}
        </div>
      ) : !wallets || wallets.length === 0 ? (
        <Card className="border-dashed py-20">
          <CardContent className="flex flex-col items-center text-center">
            <div className="bg-muted p-4 rounded-full mb-4">
              <WalletIcon className="h-10 w-10 text-muted-foreground/40" />
            </div>
            <h3 className="text-lg font-medium">No wallets found</h3>
            <p className="text-sm text-muted-foreground mt-1 max-w-xs">
              Add your first bank account or wallet to start tracking where your money goes.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {wallets.map((wallet) => (
            <WalletCard key={wallet.id} wallet={wallet} />
          ))}
        </div>
      )}
    </div>
  );
}

function WalletCard({ wallet }: { wallet: any }) {
  const typeIcon = () => {
    switch (wallet.type) {
      case "BANK":
        return <Building2Icon className="h-5 w-5" />;
      case "CREDIT_CARD":
        return <CreditCardIcon className="h-5 w-5" />;
      case "UPI":
        return <SmartphoneIcon className="h-5 w-5" />;
      default:
        return <BanknoteIcon className="h-5 w-5" />;
    }
  };

  return (
    <Card className="overflow-hidden group hover:border-primary/50 transition-all shadow-sm">
      <CardHeader className="p-4 flex flex-row items-center justify-between space-y-0">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-primary/10 text-primary">{typeIcon()}</div>
          <div>
            <CardTitle className="text-base">{wallet.name}</CardTitle>
            <CardDescription className="text-[10px] uppercase font-semibold tracking-wider">
              {wallet.type.replace("_", " ")}
            </CardDescription>
          </div>
        </div>
        <Button variant="ghost" size="icon-xs" className="opacity-0 group-hover:opacity-100 transition-opacity">
          <MoreVerticalIcon className="h-4 w-4 text-muted-foreground" />
        </Button>
      </CardHeader>
      <CardContent className="p-4 pt-0">
        <div className="mt-2">
          <p className="text-xs text-muted-foreground uppercase font-medium">Current Balance</p>
          <p className="text-2xl font-bold font-mono tracking-tight">
            ₹{parseFloat(wallet.balance).toLocaleString("en-IN", { minimumFractionDigits: 2 })}
          </p>
        </div>
      </CardContent>
      <CardFooter className="p-4 pt-0 flex gap-2">
        <Badge variant="outline" className="text-[10px] bg-muted/30">
          Last updated {new Date(wallet.updated_at).toLocaleDateString()}
        </Badge>
      </CardFooter>
    </Card>
  );
}

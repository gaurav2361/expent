"use client";

import * as React from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@expent/ui/components/dialog";
import { Button } from "@expent/ui/components/button";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "@expent/ui/components/goey-toaster";
import { SearchIcon, PlusIcon, UserIcon, WalletIcon } from "lucide-react";

import { apiClient } from "@/lib/api-client";

interface ManualTransactionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ManualTransactionDialog({ open, onOpenChange }: ManualTransactionDialogProps) {
  const queryClient = useQueryClient();
  const [amount, setAmount] = React.useState("");
  const [description, setDescription] = React.useState("");
  const [direction, setDirection] = React.useState<"IN" | "OUT">("OUT");
  const [date, setDate] = React.useState(new Date().toISOString().split("T")[0]);
  const [walletId, setWalletId] = React.useState<string>("none");
  const [contactId, setContactId] = React.useState<string>("none");

  const { data: wallets } = useQuery({
    queryKey: ["wallets"],
    queryFn: () => apiClient<any[]>("/api/wallets"),
    enabled: open,
  });

  const { data: contacts } = useQuery({
    queryKey: ["contacts"],
    queryFn: () => apiClient<any[]>("/api/contacts"),
    enabled: open,
  });

  const createMutation = useMutation({
    mutationFn: () =>
      apiClient("/api/transactions/manual", {
        method: "POST",
        body: JSON.stringify({
          amount: parseFloat(amount),
          purpose_tag: description,
          direction,
          date: new Date(date).toISOString(),
          source_wallet_id: direction === "OUT" && walletId !== "none" ? walletId : null,
          destination_wallet_id: direction === "IN" && walletId !== "none" ? walletId : null,
          contact_id: contactId !== "none" ? contactId : null,
        }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      onOpenChange(false);
      setAmount("");
      setDescription("");
      setWalletId("none");
      setContactId("none");
      toast.success("Transaction added!");
    },
    onError: (error: Error) => {
      toast.error(error.message);
    },
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[450px]">
        <DialogHeader>
          <DialogTitle>Add Transaction</DialogTitle>
          <DialogDescription>Manually enter a new transaction details.</DialogDescription>
        </DialogHeader>
        <div className="grid gap-5 py-4">
          <div className="grid gap-2">
            <Label htmlFor="amount">Amount (₹)</Label>
            <Input
              id="amount"
              type="number"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="text-lg font-mono"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="description">Description / Note</Label>
            <Input
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What was this for?"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="direction">Type</Label>
              <Select value={direction} onValueChange={(v: any) => setDirection(v)}>
                <SelectTrigger id="direction">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="OUT">Expense</SelectItem>
                  <SelectItem value="IN">Income</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="date">Date</Label>
              <Input id="date" type="date" value={date} onChange={(e) => setDate(e.target.value)} />
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="wallet">Wallet / Account</Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <WalletIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Select value={walletId} onValueChange={setWalletId}>
                  <SelectTrigger className="pl-9">
                    <SelectValue placeholder="Select wallet" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">No Wallet</SelectItem>
                    {wallets?.map((w) => (
                      <SelectItem key={w.id} value={w.id}>
                        {w.name} (₹{parseFloat(w.balance).toLocaleString()})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <Button variant="outline" size="icon" title="Add Wallet">
                <PlusIcon className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="contact">Contact / Person</Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <UserIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Select value={contactId} onValueChange={setContactId}>
                  <SelectTrigger className="pl-9">
                    <SelectValue placeholder="Select contact" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">No Contact</SelectItem>
                    {contacts?.map((c) => (
                      <SelectItem key={c.id} value={c.id}>
                        {c.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <Button variant="outline" size="icon" title="Add Contact">
                <PlusIcon className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => createMutation.mutate()}
            disabled={!amount || !description || createMutation.isPending}
          >
            {createMutation.isPending ? "Adding..." : "Add Transaction"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

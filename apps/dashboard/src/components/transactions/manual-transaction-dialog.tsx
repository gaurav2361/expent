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
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "@expent/ui/components/goey-toaster";

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

  const createMutation = useMutation({
    mutationFn: () =>
      apiClient("/api/transactions/manual", {
        method: "POST",
        body: JSON.stringify({
          amount: parseFloat(amount),
          purpose_tag: description,
          direction,
          date: new Date(date).toISOString(),
        }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      onOpenChange(false);
      setAmount("");
      setDescription("");
      toast.success("Transaction added!");
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Add Transaction</DialogTitle>
          <DialogDescription>Manually enter a new transaction details.</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="amount">Amount (₹)</Label>
            <Input
              id="amount"
              type="number"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="description">Description</Label>
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
                  <SelectValue placeholder="Select type" />
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

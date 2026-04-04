"use client";

import * as React from "react";
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
  DrawerFooter,
} from "@expent/ui/components/drawer";
import { Button } from "@expent/ui/components/button";
import { Badge } from "@expent/ui/components/badge";
import { Separator } from "@expent/ui/components/separator";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { WalletIcon } from "lucide-react";
import { useIsMobile } from "@expent/ui/hooks/use-mobile";
import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import type { Transaction, TransactionWithDetail } from "@expent/types";

interface TransactionViewerProps {
  item: TransactionWithDetail;
  onUpdate: (id: string, data: Partial<Transaction>) => void;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function TransactionViewer({ item, onUpdate, open, onOpenChange }: TransactionViewerProps) {
  const isMobile = useIsMobile();
  const [source, setSource] = React.useState<string>(item.source);
  const [category, setCategory] = React.useState(item.purpose_tag || "Uncategorized");
  const [status, setStatus] = React.useState<string>(item.status || "COMPLETED");
  const [amount, setAmount] = React.useState(item.amount);
  const [note, setNote] = React.useState(item.notes || "");

  const { data: categories } = useQuery({
    queryKey: ["categories"],
    queryFn: () => apiClient<any[]>("/api/categories"),
  });

  const title = source || "Unknown Source";
  const formattedDate = new Date(item.date).toLocaleDateString("en-IN", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <Drawer direction={isMobile ? "bottom" : "right"} open={open} onOpenChange={onOpenChange}>
      <DrawerTrigger asChild>
        <Button
          variant="link"
          className="w-fit px-0 text-left text-foreground truncate max-w-[200px] block font-normal"
        >
          {title}
        </Button>
      </DrawerTrigger>
      <DrawerContent className={isMobile ? "h-[80vh]" : "h-full w-[400px] ml-auto top-0"}>
        <DrawerHeader className="gap-1 text-left">
          <DrawerTitle className="text-xl">{title}</DrawerTitle>
          <DrawerDescription>Transaction from {formattedDate}</DrawerDescription>
        </DrawerHeader>
        <div className="flex flex-col gap-4 overflow-y-auto px-4 text-sm mt-4">
          <div className="flex items-center justify-between p-4 bg-muted rounded-xl border">
            <div>
              <div className="text-sm text-muted-foreground">Amount</div>
              <div className={`text-2xl font-bold tracking-tight ${item.direction === "IN" ? "text-green-600" : ""}`}>
                {item.direction === "OUT" ? "-" : "+"}₹
                {parseFloat(item.amount).toLocaleString("en-IN", {
                  minimumFractionDigits: 2,
                  maximumFractionDigits: 2,
                })}
              </div>
            </div>
            <Badge variant={item.direction === "IN" ? "default" : "secondary"}>
              {item.direction === "IN" ? "Income" : "Expense"}
            </Badge>
          </div>

          {(item.source_wallet_name || item.destination_wallet_name) && (
            <div className="flex flex-col gap-1 px-1">
              <span className="text-[10px] uppercase font-bold text-muted-foreground tracking-wider">
                Account / Wallet
              </span>
              <div className="flex items-center gap-2 text-sm">
                <WalletIcon className="h-4 w-4 text-primary" />
                <span>{item.source_wallet_name || item.destination_wallet_name}</span>
              </div>
            </div>
          )}

          <Separator className="my-2" />

          <form
            className="flex flex-col gap-4"
            onSubmit={(e) => {
              e.preventDefault();
              onUpdate(item.id, {
                source: source as any,
                purpose_tag: category === "Uncategorized" ? note : category,
                status: status as any,
                amount,
                notes: note,
              });
            }}
          >
            <div className="flex flex-col gap-3">
              <Label htmlFor="source">Source / Description</Label>
              <Input id="source" value={source} onChange={(e) => setSource(e.target.value)} />
            </div>

            <div className="flex flex-col gap-3">
              <Label htmlFor="amount">Amount</Label>
              <Input id="amount" type="number" step="0.01" value={amount} onChange={(e) => setAmount(e.target.value)} />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-3">
                <Label htmlFor="category">Category</Label>
                <Select value={category} onValueChange={(val) => setCategory(val || "Uncategorized")}>
                  <SelectTrigger id="category" className="w-full">
                    <SelectValue placeholder="Select a category" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Uncategorized">Uncategorized</SelectItem>
                    {categories?.map((cat) => (
                      <SelectItem key={cat.id} value={cat.name}>
                        {cat.name}
                      </SelectItem>
                    ))}
                    {!categories && (
                      <>
                        <SelectItem value="Food">Food & Drinks</SelectItem>
                        <SelectItem value="Travel">Travel</SelectItem>
                        <SelectItem value="Shopping">Shopping</SelectItem>
                        <SelectItem value="Salary">Salary</SelectItem>
                      </>
                    )}
                  </SelectContent>
                </Select>
              </div>

              <div className="flex flex-col gap-3">
                <Label htmlFor="status">Status</Label>
                <Select value={status} onValueChange={(val) => setStatus(val || "COMPLETED")}>
                  <SelectTrigger id="status" className="w-full">
                    <SelectValue placeholder="Select status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="COMPLETED">Completed</SelectItem>
                    <SelectItem value="PENDING">Pending Review</SelectItem>
                    <SelectItem value="CANCELLED">Cancelled</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="flex flex-col gap-3 mt-2">
              <Label htmlFor="note">Personal Note</Label>
              <Input
                id="note"
                value={note}
                onChange={(e) => setNote(e.target.value)}
                placeholder="Add a note about this transaction..."
              />
            </div>

            <DrawerFooter className="mt-auto px-0 pt-6 border-t border-border/50">
              <Button type="submit">Save Changes</Button>
              <DrawerClose asChild>
                <Button variant="outline">Close</Button>
              </DrawerClose>
            </DrawerFooter>
          </form>
        </div>
      </DrawerContent>
    </Drawer>
  );
}

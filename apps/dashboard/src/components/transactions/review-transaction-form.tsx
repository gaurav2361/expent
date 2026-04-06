"use client";

import type { TypedProcessedOcr } from "@expent/types";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { CheckIcon, ReceiptIcon, TagIcon, Trash2Icon, UserIcon, WalletIcon } from "lucide-react";
import * as React from "react";
import { useCategories } from "@/hooks/use-categories";

interface ReviewTransactionFormProps {
  processedOcr: TypedProcessedOcr;
  onConfirm: (finalData: TypedProcessedOcr) => void;
  onCancel: () => void;
  isSubmitting?: boolean;
}

export function ReviewTransactionForm({ processedOcr, onConfirm, onCancel, isSubmitting }: ReviewTransactionFormProps) {
  const [amount, setAmount] = React.useState("");
  const [date, setDate] = React.useState("");
  const [description, setDescription] = React.useState("");
  const [direction, setDirection] = React.useState<"IN" | "OUT">("OUT");
  const [counterparty, setCounterparty] = React.useState("");
  const [upiId, setUpiId] = React.useState("");
  const [categoryId, setCategoryId] = React.useState<string>("none");

  const { categories } = useCategories();

  React.useEffect(() => {
    if (processedOcr.doc_type === "GPAY") {
      const d = processedOcr.data;
      setAmount(d.amount?.toString() || "");
      setDirection(d.direction === "IN" ? "IN" : "OUT");
      setCounterparty(d.counterparty_name || "");
      setUpiId(d.counterparty_upi_id || "");
      setDescription(d.counterparty_name ? `Payment to ${d.counterparty_name}` : "GPay Transaction");

      // Parse '11 Mar 2026, 1:51 pm' if possible, or just use today
      if (d.datetime_str) {
        try {
          // Simple attempt to get a YYYY-MM-DD for the input
          const parts = d.datetime_str.split(" ");
          if (parts.length >= 3) {
            const day = parts[0].padStart(2, "0");
            const monthStr = parts[1];
            const year = parts[2].replace(",", "");
            const months: Record<string, string> = {
              Jan: "01",
              Feb: "02",
              Mar: "03",
              Apr: "04",
              May: "05",
              Jun: "06",
              Jul: "07",
              Aug: "08",
              Sep: "09",
              Oct: "10",
              Nov: "11",
              Dec: "12",
            };
            const month = months[monthStr] || "01";
            setDate(`${year}-${month}-${day}`);
          } else {
            setDate(new Date().toISOString().split("T")[0]);
          }
        } catch (_e) {
          setDate(new Date().toISOString().split("T")[0]);
        }
      } else {
        setDate(new Date().toISOString().split("T")[0]);
      }
    } else {
      const d = processedOcr.data;
      setAmount(d.amount?.toString() || "");
      setDirection("OUT");
      setCounterparty(d.vendor || "");
      setDescription(d.vendor ? `Purchase at ${d.vendor}` : "Generic Receipt");
      setDate(new Date().toISOString().split("T")[0]);
    }
  }, [processedOcr]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // We send back the modified ProcessedOcr structure but updated
    const updatedData: any = { ...processedOcr.data };
    updatedData.amount = amount;
    updatedData.direction = direction;

    if (processedOcr.doc_type === "GPAY") {
      updatedData.counterparty_name = counterparty;
      updatedData.counterparty_upi_id = upiId;
    } else {
      updatedData.vendor = counterparty;
    }

    if (categoryId !== "none") {
      updatedData.category_id = categoryId;
    }

    onConfirm({
      doc_type: processedOcr.doc_type as any,
      data: updatedData,
      r2_key: (processedOcr as any).r2_key,
    });
  };

  return (
    <Card className="w-full max-w-2xl mx-auto shadow-lg border-primary/10 overflow-hidden">
      <CardHeader className="bg-primary/5 border-b">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
            <ReceiptIcon className="h-5 w-5" />
          </div>
          <div>
            <CardTitle>Review Extracted Data</CardTitle>
            <CardDescription>
              Confirm the details from your {processedOcr.doc_type === "GPAY" ? "GPay screenshot" : "receipt"}.
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <form onSubmit={handleSubmit}>
        <CardContent className="grid gap-6 p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="amount">Amount (₹)</Label>
              <Input
                id="amount"
                name="amount"
                type="number"
                step="0.01"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="font-mono text-lg font-bold"
                required
                autoComplete="off"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="date">Date</Label>
              <Input
                id="date"
                name="date"
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                required
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="counterparty">{processedOcr.doc_type === "GPAY" ? "Counterparty" : "Vendor"}</Label>
            <div className="relative">
              <UserIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                id="counterparty"
                name="counterparty"
                value={counterparty}
                onChange={(e) => setCounterparty(e.target.value)}
                className="pl-9"
                placeholder="Name"
                required
                autoComplete="name"
              />
            </div>
          </div>

          {processedOcr.doc_type === "GPAY" && (
            <div className="space-y-2">
              <Label htmlFor="upiId">UPI ID / Phone</Label>
              <div className="relative">
                <WalletIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  id="upiId"
                  name="upiId"
                  value={upiId}
                  onChange={(e) => setUpiId(e.target.value)}
                  className="pl-9 font-mono text-xs"
                  placeholder="e.g. name@upi or +91…"
                />
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="direction">Direction</Label>
              <Select value={direction} onValueChange={(v: any) => setDirection(v)}>
                <SelectTrigger id="direction">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="OUT">Expense (Out)</SelectItem>
                  <SelectItem value="IN">Income (In)</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="category">Category</Label>
              <div className="relative">
                <TagIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Select value={categoryId} onValueChange={(val) => setCategoryId(val || "none")}>
                  <SelectTrigger className="pl-9">
                    {categoryId === "none" ? (
                      <span className="text-muted-foreground">Select category</span>
                    ) : (
                      <span className="truncate">
                        {categories?.find((c) => c.id === categoryId)?.name || "Unknown Category"}
                      </span>
                    )}
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">Uncategorized</SelectItem>
                    {categories?.map((c) => (
                      <SelectItem key={c.id} value={c.id}>
                        {c.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2 lg:col-span-2">
              <Label htmlFor="description">Personal Note</Label>
              <Input
                id="description"
                name="description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Optional tag or note"
                autoComplete="off"
              />
            </div>
          </div>
        </CardContent>
        <CardFooter className="bg-muted/30 border-t p-4 flex justify-between gap-3">
          <Button type="button" variant="ghost" onClick={onCancel} className="text-muted-foreground">
            <Trash2Icon className="h-4 w-4 mr-2" /> Discard
          </Button>
          <div className="flex gap-3">
            <Button type="button" variant="outline" onClick={onCancel}>
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? (
                "Saving…"
              ) : (
                <>
                  <CheckIcon className="h-4 w-4 mr-2" /> Confirm & Save
                </>
              )}
            </Button>
          </div>
        </CardFooter>
      </form>
    </Card>
  );
}

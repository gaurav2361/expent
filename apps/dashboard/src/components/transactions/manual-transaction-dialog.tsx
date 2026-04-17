"use client";

import { Button } from "@expent/ui/components/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@expent/ui/components/dialog";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { PlusIcon, TagIcon, UserIcon, WalletIcon } from "lucide-react";
import * as React from "react";
import { CreateCategoryDialog, ICON_MAP } from "@/components/categories/create-category-dialog";
import { CreateContactDialog } from "@/components/contacts/create-contact-dialog";
import { CreateWalletDialog } from "@/components/wallets/create-wallet-dialog";
import { useCategories } from "@/hooks/use-categories";
import { useContacts } from "@/hooks/use-contacts";
import { useWallets } from "@/hooks/use-wallets";
import { apiClient } from "@/lib/api-client";

const getCategoryIcon = (iconName: string | null | undefined) => {
  if (!iconName) return TagIcon;
  return ICON_MAP[iconName as keyof typeof ICON_MAP] || TagIcon;
};

const getCategoryColor = (colorHex: string | null | undefined) => {
  return colorHex || "#64748b";
};

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
  const [categoryId, setCategoryId] = React.useState<string>("none");

  const [createCategoryOpen, setCreateCategoryOpen] = React.useState(false);
  const [createContactOpen, setCreateContactOpen] = React.useState(false);
  const [createWalletOpen, setCreateWalletOpen] = React.useState(false);

  const { wallets } = useWallets();
  const { contacts } = useContacts();
  const { categories } = useCategories();

  const selectedWallet = React.useMemo(() => wallets?.find((w) => w.id === walletId), [wallets, walletId]);
  const selectedContact = React.useMemo(() => contacts?.find((c) => c.id === contactId), [contacts, contactId]);

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
          category_id: categoryId !== "none" ? categoryId : null,
        }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      onOpenChange(false);
      setAmount("");
      setDescription("");
      setWalletId("none");
      setContactId("none");
      setCategoryId("none");
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
              name="amount"
              type="number"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="text-lg font-mono"
              autoComplete="off"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="description">Description / Note</Label>
            <Input
              id="description"
              name="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What was this for?"
              autoComplete="off"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="direction">Type</Label>
              <Select value={direction} onValueChange={(v: "IN" | "OUT" | null) => setDirection(v || "OUT")}>
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
                <Select value={walletId} onValueChange={(val) => setWalletId(val || "none")}>
                  <SelectTrigger className="pl-9">
                    {walletId === "none" ? (
                      <span className="text-muted-foreground">Select wallet</span>
                    ) : (
                      <span className="truncate">{selectedWallet?.name || "Unknown Wallet"}</span>
                    )}
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
              <Button
                variant="outline"
                size="icon"
                title="Add Wallet"
                aria-label="Add new wallet"
                onClick={() => setCreateWalletOpen(true)}
              >
                <PlusIcon className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="contact">Contact / Person</Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <UserIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Select value={contactId} onValueChange={(val) => setContactId(val || "none")}>
                  <SelectTrigger className="pl-9">
                    {contactId === "none" ? (
                      <span className="text-muted-foreground">Select contact</span>
                    ) : (
                      <span className="truncate">{selectedContact?.name || "Unknown Contact"}</span>
                    )}
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
              <Button
                variant="outline"
                size="icon"
                title="Add Contact"
                aria-label="Add new contact"
                onClick={() => setCreateContactOpen(true)}
              >
                <PlusIcon className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="category">Category</Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Select value={categoryId} onValueChange={(val) => setCategoryId(val || "none")}>
                  <SelectTrigger>
                    {categoryId === "none" ? (
                      <div className="flex items-center gap-2 text-muted-foreground">
                        <TagIcon className="h-4 w-4" />
                        <span>Select category</span>
                      </div>
                    ) : (
                      (() => {
                        const cat = categories?.find((c) => c.id === categoryId);
                        if (!cat) return <span className="truncate">Unknown Category</span>;
                        const Icon = getCategoryIcon(cat.icon);
                        const color = getCategoryColor(cat.color);
                        return (
                          <div className="flex items-center gap-2 truncate">
                            <div
                              className="flex size-5 items-center justify-center rounded shrink-0"
                              style={{ backgroundColor: `${color}20`, color }}
                            >
                              <Icon className="size-3" />
                            </div>
                            <span className="truncate">{cat.name}</span>
                          </div>
                        );
                      })()
                    )}
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">
                      <div className="flex items-center gap-2">
                        <TagIcon className="h-4 w-4 text-muted-foreground" />
                        <span>Uncategorized</span>
                      </div>
                    </SelectItem>
                    {categories?.map((c) => {
                      const Icon = getCategoryIcon(c.icon);
                      const color = getCategoryColor(c.color);
                      return (
                        <SelectItem key={c.id} value={c.id}>
                          <div className="flex items-center gap-2">
                            <div
                              className="flex size-6 items-center justify-center rounded shrink-0"
                              style={{ backgroundColor: `${color}20`, color }}
                            >
                              <Icon className="size-3.5" />
                            </div>
                            <span>{c.name}</span>
                          </div>
                        </SelectItem>
                      );
                    })}
                  </SelectContent>
                </Select>
              </div>
              <Button
                variant="outline"
                size="icon"
                title="Add Category"
                aria-label="Add new category"
                onClick={() => setCreateCategoryOpen(true)}
              >
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
            {createMutation.isPending ? "Adding…" : "Add Transaction"}
          </Button>
        </DialogFooter>
      </DialogContent>
      <CreateCategoryDialog
        open={createCategoryOpen}
        onOpenChange={setCreateCategoryOpen}
        onCreated={(id) => setCategoryId(id)}
      />
      <CreateContactDialog
        open={createContactOpen}
        onOpenChange={setCreateContactOpen}
        onCreated={(id) => setContactId(id)}
      />
      <CreateWalletDialog
        open={createWalletOpen}
        onOpenChange={setCreateWalletOpen}
        onCreated={(id) => setWalletId(id)}
      />
    </Dialog>
  );
}

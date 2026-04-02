"use client";

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
import { Input } from "@expent/ui/components/input";
import { Separator } from "@expent/ui/components/separator";
import { SidebarTrigger } from "@expent/ui/components/sidebar";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@expent/ui/components/table";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { ReceiptTextIcon, Share2Icon, SparklesIcon, MoreVerticalIcon, Trash2Icon } from "lucide-react";
import { useEffect, useMemo, useState, useCallback } from "react";
import { SplitDialog } from "@/components/split-dialog";
import { useSession } from "@/lib/auth-client";
import { toast } from "@expent/ui/components/goey-toaster";
import { DataTable } from "@/components/data-table/data-table";
import type { Column } from "@/components/data-table/data-table-types";
import { TransactionViewer } from "@/components/transaction-viewer";
import type { Transaction as TransactionType } from "@/components/transaction-viewer";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@expent/ui/components/dropdown-menu";

// Replace Vite's import.meta.env with Next.js process.env
const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080";

interface OcrItem {
  name: string;
  quantity: number;
  price: string;
}

interface OcrResult {
  id: string;
  items?: OcrItem[];
  raw_text?: string;
}

interface P2PRequest {
  id: string;
  status: string;
  sender_user_id: string;
  transaction_data: {
    group_name?: string;
    amount?: string;
  };
}

export default function DashboardPage() {
  const router = useRouter();
  const session = useSession();
  const queryClient = useQueryClient();

  const [file, setFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [ocrResult, setOcrResult] = useState<OcrResult | null>(null);
  const [splitDialogOpen, setSplitDialogOpen] = useState(false);
  const [selectedTxn, setSelectedTxn] = useState<{ id: string; amount: string } | null>(null);

  const { data: transactions, isLoading: isTxnsLoading } = useQuery({
    queryKey: ["transactions"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/transactions`, {
        headers: { "Content-Type": "application/json" },
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch transactions");
      return response.json();
    },
    enabled: !!session.data,
  });

  const { data: p2pRequests } = useQuery({
    queryKey: ["p2p-pending"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/p2p/pending`, {
        headers: { "Content-Type": "application/json" },
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch P2P requests");
      return response.json();
    },
    enabled: !!session.data,
  });

  const acceptMutation = useMutation({
    mutationFn: async (requestId: string) => {
      const response = await fetch(`${API_BASE_URL}/api/p2p/accept`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ request_id: requestId }),
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to accept request");
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["p2p-pending"] });
      toast.success("Request accepted!");
    },
    onError: (error) => {
      console.error(error);
      toast.error("Failed to accept request.");
    },
  });
  
  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<TransactionType> }) => {
      const response = await fetch(`${API_BASE_URL}/api/transactions/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          amount: data.amount,
          date: data.date,
          purpose_tag: data.category || data.source,
          status: data.status,
        }),
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to update transaction");
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      toast.success("Transaction updated");
    },
    onError: (error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      const response = await fetch(`${API_BASE_URL}/api/transactions/${id}`, {
        method: "DELETE",
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to delete transaction");
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      toast.success("Transaction deleted");
    },
    onError: (error) => toast.error(error.message),
  });

  const totalBalance = useMemo(() => {
    if (!transactions) return 0;
    return transactions.reduce((acc: number, txn: TransactionType) => {
      const amount = parseFloat(txn.amount);
      return txn.direction === "IN" ? acc + amount : acc - amount;
    }, 0);
  }, [transactions]);

  const triggerSplit = useCallback((id: string, amount: string) => {
    setSelectedTxn({ id, amount });
    setSplitDialogOpen(true);
  }, []);

  const txnColumns = useMemo<Column<TransactionType>[]>(() => [
    {
      key: "date",
      label: "Date",
      format: { kind: "date", dateFormat: "short" },
    },
    {
      key: "direction",
      label: "Direction",
      format: {
        kind: "badge",
        colorMap: { IN: "success", OUT: "danger" },
      },
    },
    {
      key: "amount",
      label: "Amount",
      format: { kind: "currency", currency: "INR" },
      align: "right",
    },
    {
      key: "source",
      label: "Description",
    },
    {
      key: "action" as keyof TransactionType,
      label: " ",
      sortable: false,
      align: "right",
    },
  ] as Column<TransactionType>[], []);

  const txnCellRenderers = useMemo(() => ({
    source: (row: TransactionType) => (
      <TransactionViewer
        item={row}
        onUpdate={(id, data) => updateMutation.mutate({ id, data })}
      />
    ),
    action: (row: TransactionType) => (
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <MoreVerticalIcon className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-40">
          <DropdownMenuItem onClick={() => triggerSplit(row.id, row.amount)}>
            <Share2Icon className="mr-2 h-4 w-4" /> Split
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            variant="destructive"
            onClick={() => {
              if (confirm("Are you sure you want to delete this transaction?")) {
                deleteMutation.mutate(row.id);
              }
            }}
          >
            <Trash2Icon className="mr-2 h-4 w-4" /> Delete
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    ),
  }), [triggerSplit, updateMutation, deleteMutation]);

  useEffect(() => {
    // Navigation is primarily handled by Next.js edge middleware.
    // However, if session gets invalidated strictly client side, redirect here as well.
    if (!session.isPending && !session.data) {
      router.push("/sign-in");
    }
  }, [session.data, session.isPending, router]);

  if (session.isPending || !session.data) {
    return <div className="flex h-screen items-center justify-center">Loading session...</div>;
  }

  const handleUpload = async () => {
    if (!file) return;
    setIsUploading(true);
    const toastId = toast("Uploading and processing file...");
    try {
      const formData = new FormData();
      formData.append("file", file);

      const uploadRes = await fetch(`${API_BASE_URL}/api/upload`, {
        method: "POST",
        body: formData,
        credentials: "include",
      });

      if (!uploadRes.ok) throw new Error("Upload failed");
      const { key } = await uploadRes.json();

      const processRes = await fetch(`${API_BASE_URL}/api/process-image-ocr`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ key }),
        credentials: "include",
      });

      if (!processRes.ok) throw new Error("Processing failed");
      const result = await processRes.json();
      setOcrResult(result);
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      toast.update(toastId, { title: "File processed successfully!", type: "success" });
    } catch (error) {
      console.error(error);
      toast.update(toastId, { title: "Upload or processing failed. Please try again.", type: "error" });
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2">
        <div className="flex items-center gap-2 px-4">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
          <Breadcrumb>
            <BreadcrumbList>
              <BreadcrumbItem className="hidden md:block">
                <BreadcrumbLink href="/">Dashboard</BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator className="hidden md:block" />
              <BreadcrumbItem>
                <BreadcrumbPage>Overview</BreadcrumbPage>
              </BreadcrumbItem>
            </BreadcrumbList>
          </Breadcrumb>
        </div>
      </header>

      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Pending Approvals</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{p2pRequests?.length || 0}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Quick Upload (Images, PDF, CSV)</CardTitle>
            </CardHeader>
            <CardContent className="flex gap-2">
              <Input
                type="file"
                accept="image/*,application/pdf,text/csv"
                onChange={(e) => setFile(e.target.files?.[0] || null)}
                className="h-8 text-xs"
              />
              <Button onClick={handleUpload} disabled={!file || isUploading} size="sm">
                {isUploading ? "..." : "Go"}
              </Button>
            </CardContent>
          </Card>
          <Card
            className={totalBalance < 0 
              ? "bg-rose-50 dark:bg-rose-500/10 text-rose-600 dark:text-rose-400 shadow-lg border-rose-100 dark:border-rose-500/20" 
              : totalBalance > 0 
                ? "bg-emerald-50 dark:bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 shadow-lg border-emerald-100 dark:border-emerald-500/20"
                : "bg-muted/50 text-muted-foreground shadow-lg"
              }
          >
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Net Balance</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {totalBalance < 0 ? "-₹" : "₹"} {Math.abs(totalBalance).toLocaleString(undefined, { minimumFractionDigits: 2 })}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* OCR Result / Itemized Parsing Section */}
        {ocrResult && (
          <Card className="border-primary/20 bg-primary/5 animate-in zoom-in-95 duration-300">
            <CardHeader className="flex flex-row items-center gap-4">
              <div className="bg-primary/10 p-2 rounded-lg">
                <ReceiptTextIcon className="h-6 w-6 text-primary" />
              </div>
              <div>
                <CardTitle>Receipt Itemized</CardTitle>
                <CardDescription>We've extracted items and data from your upload.</CardDescription>
              </div>
              <Button variant="ghost" size="sm" className="ml-auto" onClick={() => setOcrResult(null)}>
                Dismiss
              </Button>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {ocrResult.items && ocrResult.items.length > 0 && (
                  <div className="rounded-md border bg-background">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>Item</TableHead>
                          <TableHead className="text-right">Qty</TableHead>
                          <TableHead className="text-right">Price</TableHead>
                          <TableHead className="text-right">Action</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {ocrResult.items.map((item) => (
                          <TableRow key={item.name}>
                            <TableCell className="font-medium">{item.name}</TableCell>
                            <TableCell className="text-right">{item.quantity}</TableCell>
                            <TableCell className="text-right font-mono">₹{item.price}</TableCell>
                            <TableCell className="text-right">
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => triggerSplit(ocrResult.id, item.price || "0")}
                              >
                                <Share2Icon className="h-3 w-3 mr-1" /> Split
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                )}
                <div className="bg-muted/50 p-3 rounded-lg text-xs font-mono whitespace-pre-wrap overflow-auto max-h-40">
                  {ocrResult.raw_text}
                </div>
                <div className="flex justify-end gap-2">
                  <Button variant="outline" size="sm">
                    <SparklesIcon className="h-3 w-3 mr-1" /> Auto-Categorize
                  </Button>
                  <Button size="sm">Confirm All</Button>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Pending P2P Requests Section */}
        {p2pRequests && p2pRequests.length > 0 && (
          <Card className="border-orange-200 bg-orange-50 dark:bg-orange-950/20 animate-in fade-in slide-in-from-top-4 duration-500">
            <CardHeader>
              <CardTitle className="text-orange-800 dark:text-orange-300 text-base">
                Action Required: P2P Requests
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {(p2pRequests as P2PRequest[]).map((req) => (
                  <div
                    key={req.id}
                    className="flex items-center justify-between border-b border-orange-100 dark:border-orange-900/50 pb-2 last:border-0"
                  >
                    <div>
                      <p className="font-semibold text-sm">
                        {req.status === "GROUP_INVITE"
                          ? "Group Invitation"
                          : `${req.sender_user_id} shared a transaction`}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {req.status === "GROUP_INVITE"
                          ? `You've been invited to join ${req.transaction_data.group_name}`
                          : `Amount: ₹${parseFloat(req.transaction_data.amount || "0").toLocaleString()}`}
                      </p>
                    </div>
                    <Button
                      size="sm"
                      variant="outline"
                      className="border-orange-300 hover:bg-orange-100"
                      onClick={() => acceptMutation.mutate(req.id)}
                    >
                      {acceptMutation.isPending
                        ? "..."
                        : req.status === "GROUP_INVITE"
                          ? "Join Group"
                          : "Merge & Accept"}
                    </Button>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Recent Transactions Table */}
        <Card className="flex-1 overflow-hidden">
          <CardHeader className="px-6 py-4 flex flex-row items-center justify-between">
            <CardTitle>Recent Transactions</CardTitle>
            <Button variant="outline" size="sm">
              View All
            </Button>
          </CardHeader>
          <CardContent className="p-0">
            {isTxnsLoading ? (
              <div className="text-center py-10 text-muted-foreground">Loading transactions...</div>
            ) : (
              <DataTable<TransactionType>
                id="dashboard-recent-transactions"
                columns={txnColumns}
                data={(transactions as TransactionType[]) ?? []}
                rowIdKey="id"
                defaultSort={{ by: "date", direction: "desc" }}
                emptyMessage="No transactions found. Start by uploading a receipt!"
                cellRenderers={txnCellRenderers}
                locale="en-IN"
              />
            )}
          </CardContent>
        </Card>
      </div>

      {selectedTxn && (
        <SplitDialog
          open={splitDialogOpen}
          onOpenChange={setSplitDialogOpen}
          transactionId={selectedTxn.id}
          totalAmount={selectedTxn.amount || "0"}
        />
      )}
    </>
  );
}

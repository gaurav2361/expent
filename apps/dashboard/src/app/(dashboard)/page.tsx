"use client";

import type { P2PRequestWithSender, TransactionWithDetail, TypedProcessedOcr } from "@expent/types";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardHeader, CardTitle } from "@expent/ui/components/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@expent/ui/components/dropdown-menu";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@expent/ui/components/tabs";
import { useQueryClient } from "@tanstack/react-query";
import {
  ActivityIcon,
  CreditCardIcon,
  FileTextIcon,
  MoreVerticalIcon,
  PlusIcon,
  Share2Icon,
  Trash2Icon,
  WalletIcon,
} from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { useCallback, useMemo, useState } from "react";
import { Analytics } from "@/components/dashboard/analytics";
import { CategoryChart } from "@/components/dashboard/category-chart";
import { IncomeExpenseChart } from "@/components/dashboard/income-expense-chart";
import { Overview } from "@/components/dashboard/overview";
import { DataTable } from "@/components/data-table/data-table";
import { ApprovalCard } from "@/components/tool-ui/approval-card";
import { ProgressTracker } from "@/components/tool-ui/progress-tracker";
import { ManualTransactionDialog } from "@/components/transactions/manual-transaction-dialog";
import { ReviewTransactionForm } from "@/components/transactions/review-transaction-form";
import { SplitDialog } from "@/components/transactions/split-dialog";
import { TransactionViewer } from "@/components/transactions/transaction-viewer";
import { useP2P } from "@/hooks/use-p2p";
import { useTransactions } from "@/hooks/use-transactions";
import { apiClient } from "@/lib/api-client";
import type { Column } from "@/lib/data-table-types";

export default function DashboardPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const queryClient = useQueryClient();

  const activeTab = searchParams.get("tab") || "overview";

  const handleTabChange = useCallback(
    (value: string | number | null) => {
      const tab = String(value);
      const params = new URLSearchParams(searchParams.toString());
      if (tab === "overview") {
        params.delete("tab");
      } else {
        params.set("tab", tab);
      }
      const qs = params.toString();
      router.replace(qs ? `/?${qs}` : "/", { scroll: false });
    },
    [router, searchParams],
  );

  const { transactions, isLoading: isTxnsLoading, updateMutation, deleteMutation } = useTransactions();
  const { p2pRequests, acceptMutation } = useP2P();

  const [file, setFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadSteps, setUploadSteps] = useState<
    { id: string; label: string; status: "pending" | "in-progress" | "completed" | "failed" }[]
  >([]);
  const [processedOcr, setProcessedOcr] = useState<TypedProcessedOcr | null>(null);
  const [isSavingOcr, setIsSavingOcr] = useState(false);
  const [splitDialogOpen, setSplitDialogOpen] = useState(false);
  const [manualDialogOpen, setManualDialogOpen] = useState(false);
  const [selectedTxn, setSelectedTxn] = useState<{ id: string; amount: string } | null>(null);

  const { totalBalance, monthlySpend } = useMemo(() => {
    if (!transactions) return { totalBalance: 0, monthlySpend: 0 };

    let bal = 0;
    let spend = 0;
    const now = new Date();
    const currentMonth = now.getMonth();
    const currentYear = now.getFullYear();

    transactions.forEach((txn: TransactionWithDetail) => {
      const amount = parseFloat(txn.amount);
      if (txn.direction === "IN") {
        bal += amount;
      } else {
        bal -= amount;

        const txnDate = new Date(txn.date);
        if (txnDate.getMonth() === currentMonth && txnDate.getFullYear() === currentYear) {
          spend += amount;
        }
      }
    });

    return { totalBalance: bal, monthlySpend: spend };
  }, [transactions]);

  const triggerSplit = useCallback((id: string, amount: string) => {
    setSelectedTxn({ id, amount });
    setSplitDialogOpen(true);
  }, []);

  const txnColumns = useMemo<Column<TransactionWithDetail>[]>(
    () =>
      [
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
          key: "contact_name" as keyof TransactionWithDetail,
          label: "Contact",
        },
        {
          key: "action" as keyof TransactionWithDetail,
          label: " ",
          sortable: false,
          align: "right",
        },
      ] as Column<TransactionWithDetail>[],
    [],
  );

  const txnCellRenderers = useMemo(
    () => ({
      source: (row: TransactionWithDetail) => (
        <TransactionViewer item={row} onUpdate={(id, data) => updateMutation.mutate({ id, data })} />
      ),
      action: (row: TransactionWithDetail) => (
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button variant="ghost" size="icon" className="h-8 w-8" aria-label="Open transaction menu">
                <MoreVerticalIcon className="h-4 w-4" />
              </Button>
            }
          />
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
    }),
    [triggerSplit, updateMutation, deleteMutation],
  );

  const handleUpload = async () => {
    if (!file) return;
    setIsUploading(true);
    setProcessedOcr(null);

    const steps: { id: string; label: string; status: "pending" | "in-progress" | "completed" | "failed" }[] = [
      { id: "1", label: "Uploading file…", status: "in-progress" as const },
      { id: "2", label: "Classifying document…", status: "pending" },
      { id: "3", label: "Extracting transaction data…", status: "pending" },
    ];
    setUploadSteps(steps);

    try {
      const formData = new FormData();
      formData.append("file", file);

      const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:7878";
      const uploadRes = await fetch(`${API_BASE_URL}/api/upload`, {
        method: "POST",
        body: formData,
        credentials: "include",
      });

      if (!uploadRes.ok) {
        const errorBody = await uploadRes.text().catch(() => "Upload failed");
        throw new Error(errorBody || "Upload failed");
      }
      const { key } = await uploadRes.json();

      setUploadSteps((prev) =>
        prev.map((s) =>
          s.id === "1" ? { ...s, status: "completed" } : s.id === "2" ? { ...s, status: "in-progress" } : s,
        ),
      );

      const result = await apiClient<TypedProcessedOcr>("/api/ocr/process", {
        method: "POST",
        body: JSON.stringify({ key }),
      });

      setUploadSteps((prev) =>
        prev.map((s) =>
          s.id === "2" ? { ...s, status: "completed" } : s.id === "3" ? { ...s, status: "in-progress" } : s,
        ),
      );

      setUploadSteps((prev) => prev.map((s) => (s.id === "3" ? { ...s, status: "completed" } : s)));

      setProcessedOcr(result);
      toast.success("Data extracted successfully! Please review.");
      setTimeout(() => setIsUploading(false), 1000);
    } catch (error) {
      console.error(error);
      setUploadSteps((prev) => prev.map((s) => (s.status === "in-progress" ? { ...s, status: "failed" } : s)));
      toast.error(error instanceof Error ? error.message : "Upload or processing failed.");
      setTimeout(() => setIsUploading(false), 2000);
    }
  };

  const handleConfirmOcr = async (finalData: TypedProcessedOcr) => {
    setIsSavingOcr(true);
    try {
      const result = await apiClient<{ contact_created: boolean }>("/api/transactions/from-ocr", {
        method: "POST",
        body: JSON.stringify(finalData),
      });
      setProcessedOcr(null);
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Transaction saved successfully!");
      if (result.contact_created) {
        toast.success("New contact auto-created from receipt!");
      }
    } catch (error) {
      console.error(error);
      toast.error(error instanceof Error ? error.message : "Failed to save transaction.");
    } finally {
      setIsSavingOcr(false);
    }
  };

  return (
    <>
      <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
        <div className="flex items-center justify-between mb-2">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Overview</h1>
            <p className="text-muted-foreground text-sm">Welcome back! Here is your financial summary.</p>
          </div>
          <div className="flex items-center space-x-2">
            <Button onClick={() => setManualDialogOpen(true)}>
              <PlusIcon className="h-4 w-4 mr-2" />
              Add Transaction
            </Button>
          </div>
        </div>

        <Tabs value={activeTab} onValueChange={handleTabChange} className="space-y-4">
          <div className="w-full overflow-x-auto pb-2">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="analytics">Analytics</TabsTrigger>
            </TabsList>
          </div>

          <TabsContent value="overview" className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Total Balance</CardTitle>
                  <WalletIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">
                    {totalBalance < 0 ? "-₹" : "₹"}
                    {Math.abs(totalBalance).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">Based on global transactions</p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Monthly Spend</CardTitle>
                  <CreditCardIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">
                    ₹{monthlySpend.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">Total expenses this month</p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">Pending Approvals</CardTitle>
                  <ActivityIcon className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">{p2pRequests?.length || 0}</div>
                  <Button
                    variant="link"
                    size="sm"
                    className="px-0 h-auto text-xs"
                    onClick={() => router.push("/p2p/pending")}
                  >
                    View Requests &rarr;
                  </Button>
                </CardContent>
              </Card>

              <Card className="bg-primary/5 dark:bg-primary/10 border-primary/20">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium text-primary">Quick Receive/Upload</CardTitle>
                  <FileTextIcon className="h-4 w-4 text-primary" />
                </CardHeader>
                <CardContent className="mt-1">
                  <Label htmlFor="quick-upload" className="sr-only">
                    Quick Upload
                  </Label>
                  <div className="flex gap-2">
                    <Input
                      id="quick-upload"
                      type="file"
                      accept="image/*,application/pdf,text/csv"
                      onChange={(e) => setFile(e.target.files?.[0] || null)}
                      className="h-8 text-xs bg-background"
                      aria-label="Select file to upload"
                    />
                    <Button onClick={handleUpload} disabled={!file || isUploading} size="sm" aria-label="Upload file">
                      {isUploading ? "…" : "Go"}
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>

            {isUploading && (
              <div className="animate-in fade-in slide-in-from-top-4 duration-300">
                <ProgressTracker id="upload-progress" steps={uploadSteps} />
              </div>
            )}

            {processedOcr && (
              <div className="animate-in zoom-in-95 duration-300">
                <ReviewTransactionForm
                  processedOcr={processedOcr}
                  onConfirm={handleConfirmOcr}
                  onCancel={() => setProcessedOcr(null)}
                  isSubmitting={isSavingOcr}
                />
              </div>
            )}

            {/* Pending P2P Actions */}
            {p2pRequests && p2pRequests.length > 0 && !processedOcr && (
              <div className="flex flex-col gap-4 animate-in fade-in slide-in-from-top-4 duration-500">
                <h2 className="text-base font-semibold flex items-center gap-2 px-1">
                  <Share2Icon className="h-4 w-4 text-primary" /> Pending Approvals
                </h2>
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                  {(p2pRequests as P2PRequestWithSender[]).map((req) => (
                    <ApprovalCard
                      key={req.id}
                      id={req.id}
                      className="max-w-none"
                      title={req.status === "GROUP_INVITE" ? "Group Invitation" : "Transaction Split"}
                      description={
                        req.status === "GROUP_INVITE"
                          ? `Join "${(req.transaction_data as { group_name?: string })?.group_name || "a group"}"`
                          : `${req.sender_name || req.sender_user_id.substring(0, 8)} shared an expense with you.`
                      }
                      icon={req.status === "GROUP_INVITE" ? "users" : "receipt"}
                      metadata={[
                        {
                          key: "Amount",
                          value: `₹${parseFloat(
                            (req.transaction_data as { amount?: string })?.amount || "0",
                          ).toLocaleString()}`,
                        },
                        {
                          key: "From",
                          value: req.sender_name || req.sender_user_id.substring(0, 8),
                        },
                      ]}
                      confirmLabel={req.status === "GROUP_INVITE" ? "Join Group" : "Accept & Merge"}
                      onConfirm={() => acceptMutation.mutate(req.id)}
                    />
                  ))}
                </div>
              </div>
            )}

            <div className="grid grid-cols-1 gap-4 lg:grid-cols-7 xl:grid-cols-7 mt-4">
              <Card className="col-span-1 lg:col-span-4 max-h-[500px]">
                <CardHeader>
                  <CardTitle>Expense Overview</CardTitle>
                </CardHeader>
                <CardContent className="ps-2">
                  <Overview />
                </CardContent>
              </Card>

              <Card className="col-span-1 lg:col-span-3 flex flex-col max-h-[500px] overflow-hidden">
                <CardHeader className="px-6 py-4 flex flex-row items-center justify-between shrink-0">
                  <CardTitle>Recent Transactions</CardTitle>
                  <Button variant="link" size="sm" onClick={() => router.push("/transactions")}>
                    View All
                  </Button>
                </CardHeader>
                <CardContent className="p-0 overflow-auto flex-1">
                  {isTxnsLoading ? (
                    <div className="text-center py-10 text-muted-foreground">Loading transactions…</div>
                  ) : (
                    <DataTable<TransactionWithDetail>
                      id="dashboard-recent-transactions"
                      columns={txnColumns}
                      data={(transactions as TransactionWithDetail[])?.slice(0, 5) ?? []}
                      rowIdKey="id"
                      defaultSort={{ by: "date", direction: "desc" }}
                      emptyMessage="No transactions found."
                      cellRenderers={txnCellRenderers}
                      locale="en-IN"
                    />
                  )}
                </CardContent>
              </Card>
            </div>

            {/* Additional Overview Charts */}
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-7 mt-4">
              <Card className="col-span-1 lg:col-span-4">
                <CardHeader>
                  <CardTitle>Income vs Expense</CardTitle>
                </CardHeader>
                <CardContent>
                  <IncomeExpenseChart transactions={transactions} />
                </CardContent>
              </Card>
              <Card className="col-span-1 lg:col-span-3">
                <CardHeader>
                  <CardTitle>Spending by Category</CardTitle>
                </CardHeader>
                <CardContent>
                  <CategoryChart transactions={transactions} />
                </CardContent>
              </Card>
            </div>
          </TabsContent>
          <TabsContent value="analytics" className="space-y-4">
            <Analytics />
          </TabsContent>
        </Tabs>
      </div>

      {selectedTxn && (
        <SplitDialog
          open={splitDialogOpen}
          onOpenChange={setSplitDialogOpen}
          transactionId={selectedTxn.id}
          totalAmount={selectedTxn.amount || "0"}
        />
      )}

      <ManualTransactionDialog open={manualDialogOpen} onOpenChange={setManualDialogOpen} />
    </>
  );
}

"use client";

import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardHeader, CardTitle } from "@expent/ui/components/card";
import { Input } from "@expent/ui/components/input";
import {
  Share2Icon,
  MoreVerticalIcon,
  Trash2Icon,
  PlusIcon,
} from "lucide-react";
import { useMemo, useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import { useQueryClient } from "@tanstack/react-query";

import { SplitDialog } from "@/components/transactions/split-dialog";
import { ManualTransactionDialog } from "@/components/transactions/manual-transaction-dialog";
import { toast } from "@expent/ui/components/goey-toaster";
import { DataTable } from "@/components/data-table/data-table";
import type { Column } from "@/components/data-table/data-table-types";
import { TransactionViewer } from "@/components/transactions/transaction-viewer";
import type { Transaction as TransactionType } from "@/components/transactions/transaction-viewer";
import { ProgressTracker } from "@/components/tool-ui/progress-tracker";
import { ApprovalCard } from "@/components/tool-ui/approval-card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@expent/ui/components/dropdown-menu";

import { useTransactions } from "@/hooks/use-transactions";
import { useP2P } from "@/hooks/use-p2p";
import type { P2PRequest } from "@/hooks/use-p2p";
import { apiClient } from "@/lib/api-client";
import { ReviewTransactionForm } from "@/components/transactions/review-transaction-form";

export default function DashboardPage() {
  const router = useRouter();
  const queryClient = useQueryClient();

  const { transactions, isLoading: isTxnsLoading, updateMutation, deleteMutation } = useTransactions();
  const { p2pRequests, acceptMutation } = useP2P();

  const [file, setFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadSteps, setUploadSteps] = useState<any[]>([]);
  const [processedOcr, setProcessedOcr] = useState<{ doc_type: string; data: any } | null>(null);
  const [isSavingOcr, setIsSavingOcr] = useState(false);
  const [splitDialogOpen, setSplitDialogOpen] = useState(false);
  const [manualDialogOpen, setManualDialogOpen] = useState(false);
  const [selectedTxn, setSelectedTxn] = useState<{ id: string; amount: string } | null>(null);

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

  const txnColumns = useMemo<Column<TransactionType>[]>(
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
          key: "action" as keyof TransactionType,
          label: " ",
          sortable: false,
          align: "right",
        },
      ] as Column<TransactionType>[],
    []
  );

  const txnCellRenderers = useMemo(
    () => ({
      source: (row: TransactionType) => (
        <TransactionViewer item={row} onUpdate={(id, data) => updateMutation.mutate({ id, data })} />
      ),
      action: (row: TransactionType) => (
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button variant="ghost" size="icon" className="h-8 w-8">
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
    [triggerSplit, updateMutation, deleteMutation]
  );

  const handleUpload = async () => {
    if (!file) return;
    setIsUploading(true);
    setProcessedOcr(null);

    const steps = [
      { id: "1", label: "Uploading file...", status: "in-progress" },
      { id: "2", label: "Classifying document...", status: "pending" },
      { id: "3", label: "Extracting transaction data...", status: "pending" },
    ];
    setUploadSteps(steps);

    try {
      const formData = new FormData();
      formData.append("file", file);

      const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080";
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
          s.id === "1" ? { ...s, status: "completed" } : s.id === "2" ? { ...s, status: "in-progress" } : s
        )
      );

      const result = await apiClient<any>("/api/process-image-ocr", {
        method: "POST",
        body: JSON.stringify({ key }),
      });

      setUploadSteps((prev) =>
        prev.map((s) =>
          s.id === "2" ? { ...s, status: "completed" } : s.id === "3" ? { ...s, status: "in-progress" } : s
        )
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

  const handleConfirmOcr = async (finalData: any) => {
    setIsSavingOcr(true);
    try {
      const result = await apiClient<any>("/api/transactions/from-ocr", {
        method: "POST",
        body: JSON.stringify(finalData),
      });
      setProcessedOcr(null);
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
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
        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Pending Approvals</CardTitle>
              <Button variant="ghost" size="icon-sm" onClick={() => router.push("/p2p/pending")}>
                <MoreVerticalIcon className="h-4 w-4" />
              </Button>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{p2pRequests?.length || 0}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Quick Upload (Images, PDF, CSV)</CardTitle>
              <Button variant="ghost" size="icon-sm" onClick={() => setManualDialogOpen(true)}>
                <PlusIcon className="h-4 w-4" />
              </Button>
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
            className={
              totalBalance < 0
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
                {totalBalance < 0 ? "-₹" : "₹"}{" "}
                {Math.abs(totalBalance).toLocaleString(undefined, { minimumFractionDigits: 2 })}
              </div>
            </CardContent>
          </Card>
        </div>

        {isUploading && (
          <div className="animate-in fade-in slide-in-from-top-4 duration-300">
            <ProgressTracker id="upload-progress" steps={uploadSteps} />
          </div>
        )}

        {/* OCR Review Form Section */}
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

        {/* Pending P2P Requests Section */}
        {p2pRequests && p2pRequests.length > 0 && !processedOcr && (
          <div className="flex flex-col gap-4 animate-in fade-in slide-in-from-top-4 duration-500">
            <h2 className="text-base font-semibold flex items-center gap-2 px-1">
              <Share2Icon className="h-4 w-4 text-primary" /> Pending Approvals
            </h2>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {(p2pRequests as P2PRequest[]).map((req) => (
                <ApprovalCard
                  key={req.id}
                  id={req.id}
                  className="max-w-none"
                  title={req.status === "GROUP_INVITE" ? "Group Invitation" : "Transaction Split"}
                  description={
                    req.status === "GROUP_INVITE"
                      ? `Join "${req.transaction_data.group_name}"`
                      : `${req.sender_name || req.sender_user_id.substring(0, 8)} shared an expense with you.`
                  }
                  icon={req.status === "GROUP_INVITE" ? "users" : "receipt"}
                  metadata={[
                    {
                      key: "Amount",
                      value: `₹${parseFloat(req.transaction_data.amount || "0").toLocaleString()}`,
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

        {/* Recent Transactions Table */}
        <Card className="flex-1 overflow-hidden">
          <CardHeader className="px-6 py-4 flex flex-row items-center justify-between">
            <CardTitle>Recent Transactions</CardTitle>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" onClick={() => router.push("/transactions")}>
                View All
              </Button>
              <Button size="sm" onClick={() => setManualDialogOpen(true)}>
                <PlusIcon className="h-4 w-4 mr-1" /> Add
              </Button>
            </div>
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

      <ManualTransactionDialog open={manualDialogOpen} onOpenChange={setManualDialogOpen} />
    </>
  );
}

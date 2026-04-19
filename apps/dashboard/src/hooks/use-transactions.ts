import type { PaginatedTransactions, Transaction, TransactionWithDetail, DashboardSummary } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useLiveQuery } from "@tanstack/react-db";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { db } from "@/lib/db";

export function useTransactions(params: { limit?: number; offset?: number } = {}) {
  const session = useSession();
  const queryClient = useQueryClient();

  // Use TanStack DB for the live query
  const query = useLiveQuery(
    () => {
      let q = db.from("transactions").orderBy("date", "desc");
      if (params.limit) q = q.limit(params.limit);
      if (params.offset) q = q.offset(params.offset);
      return q.select();
    },
    {
      enabled: !!session.data,
    },
    [params.limit, params.offset, session.data],
  );

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<TransactionWithDetail> }) => {
      // 1. Send to server
      const updatedTxn = await apiClient<Transaction>(`/api/transactions/${id}`, {
        method: "PATCH",
        body: JSON.stringify(data),
      });

      // 2. Update local DB
      await db.transactions.update(updatedTxn);

      return updatedTxn;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      // 1. Send to server
      await apiClient(`/api/transactions/${id}`, {
        method: "DELETE",
      });

      // 2. Delete from local DB
      await db.transactions.delete(id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction deleted");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    transactions: query.data as TransactionWithDetail[],
    totalCount: (query as any).totalCount || 0, // TanStack DB handles total count in meta
    isLoading: query.isLoading,
    isFetching: query.isLoading, // In DB mode, loading is fetching
    error: query.isError ? query.state.error : null,
    updateMutation,
    deleteMutation,
  };
}

export function useTransactionSummary() {
  const session = useSession();

  const query = useQuery({
    queryKey: ["transaction-summary"],
    queryFn: () => apiClient<DashboardSummary>("/api/transactions/summary"),
    enabled: !!session.data,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  return {
    summary: query.data,
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error,
    refetch: query.refetch,
  };
}

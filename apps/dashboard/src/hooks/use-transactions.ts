import type { PaginatedTransactions, Transaction, TransactionWithDetail, DashboardSummary } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useTransactions(params: { limit?: number; offset?: number } = {}) {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["transactions", params],
    queryFn: () => {
      const searchParams = new URLSearchParams();
      if (params.limit) searchParams.append("limit", params.limit.toString());
      if (params.offset) searchParams.append("offset", params.offset.toString());
      const queryString = searchParams.toString();
      return apiClient<PaginatedTransactions>(`/api/transactions${queryString ? `?${queryString}` : ""}`);
    },
    enabled: !!session.data,
    staleTime: 1000 * 60 * 2, // 2 minutes
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<TransactionWithDetail> }) =>
      apiClient<Transaction>(`/api/transactions/${id}`, {
        method: "PATCH",
        body: JSON.stringify({
          amount: data.amount,
          date: data.date,
          purpose_tag: data.purpose_tag,
          status: data.status,
          notes: data.notes,
          category_id: data.category_id,
          source_wallet_id: data.source_wallet_id,
          destination_wallet_id: data.destination_wallet_id,
          contact_id: data.contact_id,
        }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) =>
      apiClient(`/api/transactions/${id}`, {
        method: "DELETE",
      }),
    onMutate: async (id) => {
      // Cancel any outgoing refetches (so they don't overwrite our optimistic update)
      await queryClient.cancelQueries({ queryKey: ["transactions"] });

      // Snapshot the previous value
      const previousTransactions = queryClient.getQueryData(["transactions", params]);

      // Optimistically update to the new value
      queryClient.setQueryData(["transactions", params], (old: any) => {
        if (!old) return old;
        return {
          ...old,
          items: old.items.filter((t: any) => t.id !== id),
          total_count: old.total_count - 1,
        };
      });

      // Return a context object with the snapshotted value
      return { previousTransactions };
    },
    onError: (err, id, context) => {
      // If the mutation fails, use the context returned from onMutate to roll back
      queryClient.setQueryData(["transactions", params], context?.previousTransactions);
      toast.error(err.message);
    },
    onSettled: () => {
      // Always refetch after error or success to ensure server sync
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
    },
    onSuccess: () => {
      toast.success("Transaction deleted");
    },
  });

  return {
    transactions: query.data?.items,
    totalCount: query.data?.total_count || 0,
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error,
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

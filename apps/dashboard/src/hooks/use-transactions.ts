import type { Transaction, TransactionWithDetail } from "@expent/types";
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
      return apiClient<TransactionWithDetail[]>(`/api/transactions${queryString ? `?${queryString}` : ""}`);
    },
    enabled: !!session.data,
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
      toast.success("Transaction updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) =>
      apiClient(`/api/transactions/${id}`, {
        method: "DELETE",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Transaction deleted");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    transactions: query.data,
    isLoading: query.isLoading,
    error: query.error,
    updateMutation,
    deleteMutation,
  };
}

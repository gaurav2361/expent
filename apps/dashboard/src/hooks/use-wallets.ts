import type { Wallet } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useWallets() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["wallets"],
    queryFn: () => apiClient<Wallet[]>("/api/wallets"),
    enabled: !!session.data,
  });

  const createMutation = useMutation({
    mutationFn: (data: { name: string; type: string; initial_balance: number }) =>
      apiClient<Wallet>("/api/wallets", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Wallet created");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Wallet> }) =>
      apiClient<Wallet>(`/api/wallets/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Wallet updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) =>
      apiClient(`/api/wallets/${id}`, {
        method: "DELETE",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      toast.success("Wallet deleted");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    wallets: query.data,
    isLoading: query.isLoading,
    error: query.error,
    createMutation,
    updateMutation,
    deleteMutation,
  };
}

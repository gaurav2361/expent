import type { Wallet } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation } from "@tanstack/react-query";
import { useLiveQuery } from "@tanstack/react-db";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { db } from "@/lib/db";

export function useWallets() {
  const session = useSession();

  // Use TanStack DB for the live query
  const query = useLiveQuery(
    () => db.from("wallets").select(),
    {
      enabled: !!session.data,
    },
    [session.data],
  );

  const createMutation = useMutation({
    mutationFn: async (data: { name: string; type: string; initial_balance: number }) => {
      // 1. Send to server
      const newWallet = await apiClient<Wallet>("/api/wallets", {
        method: "POST",
        body: JSON.stringify(data),
      });

      // 2. Insert into local DB (TanStack DB will react immediately)
      await db.wallets.insert(newWallet);

      return newWallet;
    },
    onSuccess: () => {
      toast.success("Wallet created");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<Wallet> }) => {
      // 1. Send to server
      const updatedWallet = await apiClient<Wallet>(`/api/wallets/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      });

      // 2. Update local DB
      await db.wallets.update(updatedWallet);

      return updatedWallet;
    },
    onSuccess: () => {
      toast.success("Wallet updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      // 1. Send to server
      await apiClient(`/api/wallets/${id}`, {
        method: "DELETE",
      });

      // 2. Delete from local DB
      await db.wallets.delete(id);
    },
    onSuccess: () => {
      toast.success("Wallet deleted");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    wallets: query.data,
    isLoading: query.isLoading,
    error: query.isError ? query.state.error : null,
    createMutation,
    updateMutation,
    deleteMutation,
  };
}

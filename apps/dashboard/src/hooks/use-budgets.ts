import type { Budget, BudgetHealth, BudgetPeriod } from "@expent/types";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useBudgets() {
  const queryClient = useQueryClient();
  const session = useSession();

  const budgetsQuery = useQuery({
    queryKey: ["budgets"],
    queryFn: () => api.get<Budget[]>("/api/budgets"),
    enabled: !!session.data,
  });

  const healthQuery = useQuery({
    queryKey: ["budgets", "health"],
    queryFn: () => api.get<BudgetHealth[]>("/api/budgets/health"),
    enabled: !!session.data,
  });

  const createMutation = useMutation({
    mutationFn: (data: { category_id?: string; amount: string; period: BudgetPeriod }) =>
      api.post<Budget>("/api/budgets", data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["budgets"] });
      queryClient.invalidateQueries({ queryKey: ["budgets", "health"] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, ...data }: { id: string; amount?: string; period?: BudgetPeriod }) =>
      api.patch<Budget>(`/api/budgets/${id}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["budgets"] });
      queryClient.invalidateQueries({ queryKey: ["budgets", "health"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/budgets/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["budgets"] });
      queryClient.invalidateQueries({ queryKey: ["budgets", "health"] });
    },
  });

  return {
    budgets: budgetsQuery.data,
    health: healthQuery.data,
    isLoading: budgetsQuery.isLoading || healthQuery.isLoading,
    createMutation,
    updateMutation,
    deleteMutation,
  };
}

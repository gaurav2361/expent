import type { Category } from "@expent/types";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useCategories() {
  const queryClient = useQueryClient();
  const session = useSession();

  const query = useQuery({
    queryKey: ["categories"],
    queryFn: () => apiClient<Category[]>("/api/categories"),
    enabled: !!session.data,
  });

  const createMutation = useMutation({
    mutationFn: (data: { name: string; icon?: string; color?: string }) =>
      apiClient<Category>("/api/categories", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) =>
      apiClient(`/api/categories/${id}`, {
        method: "DELETE",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
    },
  });

  return {
    categories: query.data,
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    createMutation,
    deleteMutation,
  };
}

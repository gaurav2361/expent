import type { Category } from "@expent/types";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useCategories() {
  const queryClient = useQueryClient();
  const session = useSession();

  const query = useQuery({
    queryKey: ["categories"],
    queryFn: () => api.get<Category[]>("/api/categories"),
    enabled: !!session.data,
    staleTime: 1000 * 60 * 60, // 1 hour
  });

  const createMutation = useMutation({
    mutationFn: (data: { name: string; icon?: string; color?: string }) => api.post<Category>("/api/categories", data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/categories/${id}`),
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

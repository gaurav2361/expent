import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { toast } from "@expent/ui/components/goey-toaster";

export interface P2PRequest {
  id: string;
  status: string;
  sender_user_id: string;
  transaction_data: any;
  sender_name?: string;
}

export function useP2P() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["p2p-pending"],
    queryFn: () => apiClient<P2PRequest[]>("/api/p2p/pending"),
    enabled: !!session.data,
  });

  const acceptMutation = useMutation({
    mutationFn: (requestId: string) =>
      apiClient("/api/p2p/accept", {
        method: "POST",
        body: JSON.stringify({ request_id: requestId }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      queryClient.invalidateQueries({ queryKey: ["p2p-pending"] });
      toast.success("Request accepted!");
    },
    onError: (error: Error) => {
      console.error(error);
      toast.error("Failed to accept request.");
    },
  });

  return {
    p2pRequests: query.data,
    isLoading: query.isLoading,
    error: query.error,
    acceptMutation,
  };
}

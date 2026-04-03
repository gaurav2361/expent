"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSession } from "@/lib/auth-client";
import { ApprovalCard } from "@/components/tool-ui/approval-card";
import { toast } from "@expent/ui/components/goey-toaster";
import { Skeleton } from "@expent/ui/components/skeleton";

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080";

interface P2PRequest {
  id: string;
  status: string;
  sender_user_id: string;
  transaction_data: any;
}

export default function PendingPage() {
  const session = useSession();
  const queryClient = useQueryClient();

  const { data: p2pRequests, isLoading } = useQuery({
    queryKey: ["p2p-pending"],
    queryFn: async () => {
      const response = await fetch(`${API_BASE_URL}/api/p2p/pending`, {
        headers: { "Content-Type": "application/json" },
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to fetch P2P requests");
      return response.json();
    },
    enabled: !!session.data,
  });

  const acceptMutation = useMutation({
    mutationFn: async (requestId: string) => {
      const response = await fetch(`${API_BASE_URL}/api/p2p/accept`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ request_id: requestId }),
        credentials: "include",
      });
      if (!response.ok) throw new Error("Failed to accept request");
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["p2p-pending"] });
      toast.success("Request accepted!");
    },
  });

  if (isLoading) {
    return (
      <div className="flex flex-1 flex-col gap-6 p-4 lg:p-8 max-w-2xl mx-auto w-full">
        <Skeleton className="h-40 w-full" />
        <Skeleton className="h-40 w-full" />
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 lg:p-8 max-w-2xl mx-auto w-full">
      <div className="flex items-center justify-between">
        <h1 className="font-semibold text-lg md:text-2xl">Pending Requests</h1>
      </div>

      {!p2pRequests || p2pRequests.length === 0 ? (
        <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed shadow-xs min-h-[400px]">
          <div className="flex flex-col items-center gap-1 text-center">
            <h3 className="font-bold text-2xl tracking-tight">No Pending Requests</h3>
            <p className="text-sm text-muted-foreground">You don't have any pending requests to approve right now.</p>
          </div>
        </div>
      ) : (
        <div className="flex flex-col gap-4">
          {p2pRequests.map((req: P2PRequest) => (
            <ApprovalCard
              key={req.id}
              id={req.id}
              title={req.status === "GROUP_INVITE" ? "Group Invitation" : "Transaction Split"}
              description={
                req.status === "GROUP_INVITE"
                  ? `You've been invited to join ${req.transaction_data.group_name}`
                  : `${req.sender_user_id} shared an expense with you.`
              }
              icon={req.status === "GROUP_INVITE" ? "users" : "receipt"}
              metadata={[
                { key: "Amount", value: `₹${parseFloat(req.transaction_data.amount || "0").toLocaleString()}` },
                { key: "From", value: req.sender_user_id.substring(0, 8) },
              ]}
              confirmLabel={req.status === "GROUP_INVITE" ? "Join Group" : "Accept Split"}
              onConfirm={() => acceptMutation.mutate(req.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

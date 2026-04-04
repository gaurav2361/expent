"use client";

import { useP2P } from "@/hooks/use-p2p";
import type { P2PRequest } from "@/hooks/use-p2p";
import { ApprovalCard } from "@/components/tool-ui/approval-card";
import { Skeleton } from "@expent/ui/components/skeleton";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { toast } from "@expent/ui/components/goey-toaster";

export default function PendingPage() {
  const { p2pRequests, isLoading, acceptMutation } = useP2P();
  const queryClient = useQueryClient();

  const rejectMutation = useMutation({
    mutationFn: (requestId: string) =>
      apiClient(`/api/p2p/reject/${requestId}`, {
        method: "POST",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["p2p-pending"] });
      toast.success("Request rejected");
    },
    onError: (error: Error) => toast.error(error.message),
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
                  : `${req.sender_name || req.sender_user_id} shared an expense with you.`
              }
              icon={req.status === "GROUP_INVITE" ? "users" : "receipt"}
              metadata={[
                { key: "Amount", value: `₹${parseFloat(req.transaction_data.amount || "0").toLocaleString()}` },
                { key: "From", value: req.sender_name || req.sender_user_id.substring(0, 8) },
              ]}
              confirmLabel={req.status === "GROUP_INVITE" ? "Join Group" : "Accept Split"}
              onConfirm={() => acceptMutation.mutate(req.id)}
              onCancel={() => {
                if (confirm("Are you sure you want to reject this request?")) {
                  rejectMutation.mutate(req.id);
                }
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}

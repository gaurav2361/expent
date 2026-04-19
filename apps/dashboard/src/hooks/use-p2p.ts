import type {
  Group,
  LedgerTab,
  P2PRequest,
  P2PRequestWithSender,
  Transaction,
  User,
  GroupMemberDetail,
} from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useP2P() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["p2p-pending"],
    queryFn: () => apiClient<P2PRequestWithSender[]>("/api/p2p/pending"),
    enabled: !!session.data,
  });

  const acceptMutation = useMutation({
    mutationFn: (requestId: string) =>
      apiClient<Transaction>("/api/p2p/accept", {
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

  return {
    p2pRequests: query.data,
    isLoading: query.isLoading,
    error: query.error,
    acceptMutation,
    rejectMutation,
  };
}

export function useGroups() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["groups"],
    queryFn: () => apiClient<Group[]>("/api/groups"),
    enabled: !!session.data,
  });

  const createMutation = useMutation({
    mutationFn: (data: { name: string; description?: string | null }) =>
      apiClient<Group>("/api/groups/create", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["groups"] });
      toast.success("Group created");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const inviteMutation = useMutation({
    mutationFn: (data: { groupId: string; email: string }) =>
      apiClient<P2PRequest>("/api/groups/invite", {
        method: "POST",
        body: JSON.stringify({ group_id: data.groupId, receiver_email: data.email }),
      }),
    onSuccess: () => {
      toast.success("Invite sent!");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    groups: query.data,
    isLoading: query.isLoading,
    createMutation,
    inviteMutation,
  };
}

export function useGroupMembers(groupId: string) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["group-members", groupId],
    queryFn: () => apiClient<GroupMemberDetail[]>(`/api/groups/${groupId}/members`),
    enabled: !!groupId,
  });

  const removeMemberMutation = useMutation({
    mutationFn: (userId: string) =>
      apiClient(`/api/groups/${groupId}/members/${userId}`, {
        method: "DELETE",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["group-members", groupId] });
      toast.success("Member removed");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateRoleMutation = useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: string }) =>
      apiClient(`/api/groups/${groupId}/members/${userId}/role`, {
        method: "PATCH",
        body: JSON.stringify({ role }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["group-members", groupId] });
      toast.success("Role updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    members: query.data,
    isLoading: query.isLoading,
    removeMemberMutation,
    updateRoleMutation,
  };
}

export function useLedgerTabs() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["ledger-tabs"],
    queryFn: () => apiClient<LedgerTab[]>("/api/p2p/ledger-tabs"),
    enabled: !!session.data,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) =>
      apiClient<LedgerTab>("/api/p2p/ledger-tabs", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ledger-tabs"] });
      toast.success("Ledger tab created");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const repaymentMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: any }) =>
      apiClient<Transaction>(`/api/p2p/ledger-tabs/${id}/repayment`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ledger-tabs"] });
      queryClient.invalidateQueries({ queryKey: ["transactions"] });
      toast.success("Repayment registered!");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    ledgerTabs: query.data,
    isLoading: query.isLoading,
    createMutation,
    repaymentMutation,
  };
}

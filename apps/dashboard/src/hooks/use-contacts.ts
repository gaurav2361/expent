import type { Contact, ContactIdentifier, Transaction } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useContacts() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contacts"],
    queryFn: () => apiClient<Contact[]>("/api/contacts"),
    enabled: !!session.data,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const createMutation = useMutation({
    mutationFn: (data: { name: string; phone?: string | null }) =>
      apiClient<Contact>("/api/contacts", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
      toast.success("Contact added");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Contact> }) =>
      apiClient<Contact>(`/api/contacts/${id}`, {
        method: "PUT",
        body: JSON.stringify(data),
      }),
    onMutate: async ({ id, data }) => {
      await queryClient.cancelQueries({ queryKey: ["contacts"] });
      const previousContacts = queryClient.getQueryData<Contact[]>(["contacts"]);

      queryClient.setQueryData<Contact[]>(["contacts"], (old) => {
        if (!old) return old;
        return old.map((c) => (c.id === id ? { ...c, ...data } : c));
      });

      return { previousContacts };
    },
    onError: (err, variables, context) => {
      queryClient.setQueryData(["contacts"], context?.previousContacts);
      toast.error(err.message);
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) =>
      apiClient(`/api/contacts/${id}`, {
        method: "DELETE",
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
      toast.success("Contact removed");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    contacts: query.data,
    isLoading: query.isLoading,
    error: query.error,
    createMutation,
    updateMutation,
    deleteMutation,
  };
}

export function useMergeContacts() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contacts-suggestions"],
    queryFn: () => apiClient<{ contacts: Contact[]; reason: string }[]>("/api/contacts/suggestions"),
    enabled: !!session.data,
  });

  const mergeMutation = useMutation({
    mutationFn: (data: { primary_id: string; secondary_id: string }) =>
      apiClient<Contact>("/api/contacts/merge", {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
      queryClient.invalidateQueries({ queryKey: ["contacts-suggestions"] });
      queryClient.invalidateQueries({ queryKey: ["contact-detail", variables.primary_id] });
      toast.success("Contacts merged successfully");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    suggestions: query.data,
    isLoading: query.isLoading,
    error: query.error,
    mergeMutation,
  };
}

export function useContactDetail(id: string) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contact-detail", id],
    queryFn: () =>
      apiClient<{
        contact: Contact;
        identifiers: ContactIdentifier[];
        transactions: Transaction[];
      }>(`/api/contacts/${id}`),
    enabled: !!id,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const addIdentifierMutation = useMutation({
    mutationFn: (data: { type: string; value: string }) =>
      apiClient<ContactIdentifier>(`/api/contacts/${id}/identifiers`, {
        method: "POST",
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contact-detail", id] });
      toast.success("Identifier added");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    contactData: query.data,
    isLoading: query.isLoading,
    addIdentifierMutation,
  };
}

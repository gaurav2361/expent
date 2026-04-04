import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { toast } from "@expent/ui/components/goey-toaster";
import type { Contact, ContactIdentifier, Transaction } from "@expent/types";

export function useContacts() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contacts"],
    queryFn: () => apiClient<Contact[]>("/api/contacts"),
    enabled: !!session.data,
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
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
    },
    onError: (error: Error) => toast.error(error.message),
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

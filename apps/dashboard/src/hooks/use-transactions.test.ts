import { renderHook, act } from "@testing-library/react-hooks";
import { useTransactions, useTransactionSummary } from "./use-transactions";
import { vi, describe, it, expect, beforeEach } from "vitest";
import { apiClient } from "@/lib/api-client";
import { toast } from "@expent/ui/components/goey-toaster";

// Mock dependencies
vi.mock("@/lib/api-client");
vi.mock("@/lib/auth-client", () => ({
  useSession: () => ({ data: { user: { id: "test-user" } } }),
}));
vi.mock("@tanstack/react-query", () => ({
  useMutation: vi.fn(({ mutationFn, onSuccess, onError }) => ({
    mutateAsync: async (variables: any) => {
      try {
        const result = await mutationFn(variables);
        if (onSuccess) onSuccess(result, variables);
        return result;
      } catch (error) {
        if (onError) onError(error);
        throw error;
      }
    },
    isLoading: false,
  })),
  useQuery: vi.fn(({ queryFn }) => {
    queryFn(); // Execute it to verify it was called
    return { data: null, isLoading: false };
  }),
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));
vi.mock("@tanstack/react-db", () => ({
  useLiveQuery: vi.fn(() => ({ data: [], isLoading: false })),
}));
vi.mock("@/lib/db", () => ({
  db: {
    transactions: {
      update: vi.fn(),
      delete: vi.fn(),
    },
  },
}));

describe("useTransactions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should handle update transaction success", async () => {
    const mockTxn = { id: "1", amount: 100 };
    (apiClient as any).mockResolvedValue(mockTxn);

    const { result } = renderHook(() => useTransactions());

    await act(async () => {
      await result.current.updateMutation.mutateAsync({ id: "1", data: { amount: 100 } });
    });

    expect(apiClient).toHaveBeenCalledWith("/api/transactions/1", expect.objectContaining({ method: "PATCH" }));
    expect(toast.success).toHaveBeenCalledWith("Transaction updated");
  });

  it("should handle update transaction error", async () => {
    const error = new Error("API Error");
    (apiClient as any).mockRejectedValue(error);

    const { result } = renderHook(() => useTransactions());

    try {
      await act(async () => {
        await result.current.updateMutation.mutateAsync({ id: "1", data: { amount: 100 } });
      });
    } catch (e) {
      // Expected
    }

    expect(toast.error).toHaveBeenCalledWith("API Error");
  });

  it("should handle delete transaction success", async () => {
    (apiClient as any).mockResolvedValue({});

    const { result } = renderHook(() => useTransactions());

    await act(async () => {
      await result.current.deleteMutation.mutateAsync("1");
    });

    expect(apiClient).toHaveBeenCalledWith("/api/transactions/1", expect.objectContaining({ method: "DELETE" }));
    expect(toast.success).toHaveBeenCalledWith("Transaction deleted");
  });
});

describe("useTransactionSummary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch summary success", async () => {
    const mockSummary = { total_balance: 100 };
    (apiClient as any).mockResolvedValue(mockSummary);

    renderHook(() => useTransactionSummary());

    expect(apiClient).toHaveBeenCalledWith("/api/transactions/summary");
  });
});

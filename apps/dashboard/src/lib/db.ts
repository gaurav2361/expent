import { createCollection, localStorageCollectionOptions } from "@tanstack/db";
import type { Wallet, Transaction, PaginatedTransactions } from "@expent/types";
import { apiClient } from "./api-client";

// In @tanstack/db v0.6.5, we export an object with collections.
// We use localStorageCollectionOptions to handle persistence and cross-tab sync.

const walletOptions = localStorageCollectionOptions({
  storageKey: "expent_wallets",
  getKey: (wallet: Wallet) => wallet.id,
});

const transactionsOptions = localStorageCollectionOptions({
  storageKey: "expent_transactions",
  getKey: (txn: Transaction) => txn.id,
});

export const db = {
  wallets: createCollection({
    ...walletOptions,
    sync: {
      sync: (params) => {
        // 1. Initial hydration from local storage
        walletOptions.sync.sync(params);

        // 2. Refresh from remote API
        apiClient<Wallet[]>("/api/wallets")
          .then((wallets) => {
            params.begin();
            for (const wallet of wallets) {
              params.write({ type: "insert", value: wallet });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync wallets:", error));
      },
    },
  }),
  transactions: createCollection({
    ...transactionsOptions,
    sync: {
      sync: (params) => {
        // 1. Initial hydration from local storage
        transactionsOptions.sync.sync(params);

        // 2. Refresh from remote API (limited to last 100 for hydration)
        apiClient<PaginatedTransactions>("/api/transactions?limit=100")
          .then((res) => {
            params.begin();
            for (const txn of res.items) {
              params.write({ type: "insert", value: txn });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync transactions:", error));
      },
    },
  }),
};

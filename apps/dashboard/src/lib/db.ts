import { createDB, createCollection, persistedCollectionOptions } from "@tanstack/db";
import { browserCollectionCoordinator } from "@tanstack/db/browser";
import type { Wallet, Transaction, PaginatedTransactions } from "@expent/types";
import { apiClient } from "./api-client";

// Coordinator for multi-tab support
const coordinator = browserCollectionCoordinator();

export const db = createDB({
  tables: {
    wallets: createCollection<Wallet, "id">({
      ...persistedCollectionOptions({
        coordinator,
        key: "expent_wallets",
      }),
      getKey: (wallet) => wallet.id,
      sync: {
        sync: async ({ begin, write, commit, markReady }) => {
          try {
            const wallets = await apiClient<Wallet[]>("/api/wallets");
            begin();
            for (const wallet of wallets) {
              write({ type: "upsert", data: wallet });
            }
            commit();
            markReady();
          } catch (error) {
            console.error("Failed to sync wallets:", error);
          }
        },
      },
    }),
    transactions: createCollection<Transaction, "id">({
      ...persistedCollectionOptions({
        coordinator,
        key: "expent_transactions",
      }),
      getKey: (txn) => txn.id,
      sync: {
        // We'll use a progressive sync for transactions
        sync: async ({ begin, write, commit, markReady }) => {
          try {
            // Pull the last 100 transactions to hydrate the local DB
            const res = await apiClient<PaginatedTransactions>("/api/transactions?limit=100");
            begin();
            for (const txn of res.items) {
              write({ type: "upsert", data: txn });
            }
            commit();
            markReady();
          } catch (error) {
            console.error("Failed to sync transactions:", error);
          }
        },
      },
    }),
  },
});

"use client";

import { useGlobalStore } from "@/lib/store";
import { ManualTransactionDialog } from "@/components/transactions/manual-transaction-dialog";

export function GlobalModals() {
  const { isTransactionModalOpen, setTransactionModalOpen } = useGlobalStore();

  return (
    <>
      <ManualTransactionDialog open={isTransactionModalOpen} onOpenChange={setTransactionModalOpen} />
    </>
  );
}

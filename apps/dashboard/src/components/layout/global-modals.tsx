"use client";

import { useGlobalStore } from "@/lib/store";
import { ManualTransactionDialog } from "@/components/transactions/manual-transaction-dialog";
import { GlobalOCRDialog } from "@/components/transactions/global-ocr-dialog";
import { CreateCategoryDialog } from "@/components/categories/create-category-dialog";

export function GlobalModals() {
  const {
    isTransactionModalOpen,
    setTransactionModalOpen,
    isOCRModalOpen,
    setOCRModalOpen,
    isCategoryModalOpen,
    setCategoryModalOpen,
  } = useGlobalStore();

  return (
    <>
      <ManualTransactionDialog open={isTransactionModalOpen} onOpenChange={setTransactionModalOpen} />
      <GlobalOCRDialog open={isOCRModalOpen} onOpenChange={setOCRModalOpen} />
      <CreateCategoryDialog open={isCategoryModalOpen} onOpenChange={setCategoryModalOpen} />
    </>
  );
}

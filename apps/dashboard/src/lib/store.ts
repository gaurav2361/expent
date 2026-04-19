import { create } from "zustand";

interface GlobalStore {
  isTransactionModalOpen: boolean;
  setTransactionModalOpen: (open: boolean) => void;
  isHotkeyHelpOpen: boolean;
  setHotkeyHelpOpen: (open: boolean) => void;
}

export const useGlobalStore = create<GlobalStore>((set) => ({
  isTransactionModalOpen: false,
  setTransactionModalOpen: (open) => set({ isTransactionModalOpen: open }),
  isHotkeyHelpOpen: false,
  setHotkeyHelpOpen: (open) => set({ isHotkeyHelpOpen: open }),
}));

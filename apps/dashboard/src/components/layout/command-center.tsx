"use client";

import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@expent/ui/components/command";
import { useHotkey, useHotkeySequence } from "@tanstack/react-hotkeys";
import {
  CalculatorIcon,
  CalendarIcon,
  LayoutDashboardIcon,
  PlusIcon,
  ReceiptIcon,
  SettingsIcon,
  UserPlusIcon,
  UsersIcon,
  WalletIcon,
} from "lucide-react";
import { useRouter } from "next/navigation";
import { useTheme } from "next-themes";
import * as React from "react";
import { useContacts } from "@/hooks/use-contacts";
import { useTransactions } from "@/hooks/use-transactions";
import { useWallets } from "@/hooks/use-wallets";

import { useGlobalStore } from "@/lib/store";

export function CommandCenter() {
  const [open, setOpen] = React.useState(false);
  const router = useRouter();
  const { setTheme } = useTheme();
  const { setTransactionModalOpen, setHotkeyHelpOpen } = useGlobalStore();

  // Fetch some data for quick search
  const { contacts } = useContacts();
  const { wallets } = useWallets();

  // Toggle Command Palette - Use object form for better compatibility
  useHotkey({ key: "K", mod: true }, (e) => {
    e.preventDefault();
    setOpen((open) => !open);
  });

  // Global Quick Shortcuts (when palette is closed)
  useHotkey("N", (e) => {
    if (open) return;
    setTransactionModalOpen(true);
  });

  // Hotkey Help - Use RawHotkey object to avoid literal type mismatch
  useHotkey({ key: "?" }, (e) => {
    if (open) return;
    setHotkeyHelpOpen(true);
  });

  // Navigation Sequences
  useHotkeySequence(["G", "D"], () => {
    if (open) return;
    router.push("/");
  });

  useHotkeySequence(["G", "T"], () => {
    if (open) return;
    router.push("/transactions");
  });

  useHotkeySequence(["G", "W"], () => {
    if (open) return;
    router.push("/wallets");
  });

  useHotkeySequence(["G", "C"], () => {
    if (open) return;
    router.push("/contacts");
  });

  const runCommand = React.useCallback((command: () => void) => {
    setOpen(false);
    React.startTransition(() => {
      command();
    });
  }, []);

  return (
    <CommandDialog
      open={open}
      onOpenChange={setOpen}
      className="bg-card/80 backdrop-blur-xl border-white/10 dark:border-white/5 shadow-2xl overflow-hidden"
    >
      <div className="absolute inset-0 bg-[url('/noise.svg')] opacity-[0.03] pointer-events-none mix-blend-overlay" />
      <CommandInput placeholder="Type a command or search..." className="h-12 border-none bg-transparent" />
      <CommandList className="max-h-[450px] relative z-10 no-scrollbar">
        <CommandEmpty>No results found.</CommandEmpty>

        <CommandGroup heading="Quick Actions" className="px-2">
          <CommandItem
            onSelect={() => runCommand(() => setTransactionModalOpen(true))}
            className="rounded-lg mb-1 data-selected:bg-primary/10 data-selected:text-primary transition-all duration-200"
          >
            <div className="size-8 rounded-md bg-primary/10 flex items-center justify-center mr-3">
              <PlusIcon className="h-4 w-4 text-primary" />
            </div>
            <span className="font-medium">Add Transaction</span>
            <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
              N
            </kbd>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => setHotkeyHelpOpen(true))}
            className="rounded-lg mb-1 transition-all duration-200"
          >
            <div className="size-8 rounded-md bg-muted flex items-center justify-center mr-3">
              <CalculatorIcon className="h-4 w-4" />
            </div>
            <span className="font-medium">View Hotkeys</span>
            <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
              ?
            </kbd>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator className="opacity-50" />

        <CommandGroup heading="Navigation" className="px-2">
          <CommandItem
            onSelect={() => runCommand(() => router.push("/"))}
            className="rounded-lg mb-1 transition-all duration-200"
          >
            <LayoutDashboardIcon className="mr-3 h-4 w-4 opacity-70" />
            <span>Dashboard</span>
            <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
              G D
            </kbd>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => router.push("/transactions"))}
            className="rounded-lg mb-1 transition-all duration-200"
          >
            <ReceiptIcon className="mr-3 h-4 w-4 opacity-70" />
            <span>Transactions</span>
            <kbd className="ml-auto pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100">
              G T
            </kbd>
          </CommandItem>
        </CommandGroup>

        <CommandSeparator className="opacity-50" />

        <CommandGroup heading="Wallets" className="px-2">
          {wallets?.map((wallet) => (
            <CommandItem
              key={wallet.id}
              onSelect={() => runCommand(() => router.push("/wallets"))}
              className="rounded-lg mb-1 transition-all duration-200"
            >
              <div className="size-8 rounded-md bg-muted/50 flex items-center justify-center mr-3">
                <WalletIcon className="h-4 w-4 opacity-70" />
              </div>
              <span>{wallet.name}</span>
              <span className="ml-auto tabular-nums text-xs text-muted-foreground">
                ₹{parseFloat(wallet.balance).toLocaleString()}
              </span>
            </CommandItem>
          ))}
        </CommandGroup>

        <CommandSeparator className="opacity-50" />

        <CommandGroup heading="Recent Contacts" className="px-2">
          {contacts?.slice(0, 5).map((contact) => (
            <CommandItem
              key={contact.id}
              onSelect={() => runCommand(() => router.push(`/contacts/${contact.id}`))}
              className="rounded-lg mb-1 transition-all duration-200"
            >
              <div className="size-8 rounded-md bg-muted/50 flex items-center justify-center mr-3">
                <UserPlusIcon className="h-4 w-4 opacity-70" />
              </div>
              <span>{contact.name}</span>
            </CommandItem>
          ))}
        </CommandGroup>

        <CommandSeparator className="opacity-50" />

        <CommandGroup heading="Settings" className="px-2">
          <CommandItem
            onSelect={() => runCommand(() => router.push("/settings/profile"))}
            className="rounded-lg mb-1 transition-all duration-200"
          >
            <SettingsIcon className="mr-3 h-4 w-4 opacity-70" />
            <span>Profile</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => setTheme("light"))}
            className="rounded-lg mb-1 transition-all duration-200"
          >
            <CalendarIcon className="mr-3 h-4 w-4 opacity-70" />
            <span>Light Mode</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => setTheme("dark"))}
            className="rounded-lg mb-1 transition-all duration-200"
          >
            <CalendarIcon className="mr-3 h-4 w-4 opacity-70" />
            <span>Dark Mode</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}

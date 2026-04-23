# Expent Dashboard Architecture & UI Documentation

This document covers the architectural layout, frontend stack, routing structure, and component methodology established for the Expent web dashboard.

## Architectural Overview

The dashboard is built within the `apps/dashboard` monorepo package. It leverages cutting-edge React ecosystem tools tailored for highly dynamic, visually heavy reporting and transaction management interfaces.

- **Framework**: Next.js 16 (Strictly utilizing the `App Router` under `src/app`).
- **Core Library**: React 19.
- **Compilation**: Turbocharged with the `babel-plugin-react-compiler` (React Compiler) for automatic memoization.
- **State Management**:
  - Server Cache / Action Fetches: `@tanstack/react-query`
  - Client Global App State: `zustand`
- **Styling**: Tailwind CSS v4 & `next-themes`.
- **Animations**: `motion`.
- **Dependencies**: Imports strongly from shared workspace packages (`@expent/ui` for UI and `@expent/types` for TypeScript bounds).
- **Authentication**: Directly wired into `better-auth` using native React bindings.

---

## Codebase Structure & Component Strategy

The codebase is logically split in the `src/` directory to handle complex React scaling cleanly by decoupling core page logic from deeply styled UI blocks.

### `src/components/`

- **`auth/`**: Contains the `AuthGuard` component.
- **`layout/`**: Holds the global UI wiring (Sidebar, Navbar, NavMain, NavUser).
- **`transactions/`**: Heavily targeted business logic forms:
  - `manual-transaction-dialog.tsx`: Form for injecting new financial actions.
  - `split-dialog.tsx`: Fractional distribution of payments.
  - `transaction-viewer.tsx`: Detailed transaction inspection.
- **`data-table/` & `tool-ui/`**: Advanced headless table components driven by `@tanstack/react-table`.

### `src/hooks/` & `src/lib/`

- **Hooks**: Houses media query and state abstractions.
- **Libs Environment**:
  - `query-client.ts`: Global React Query logic.
  - `auth-client.ts`: `better-auth/react` client with `passkeyClient()` and `usernameClient()`.
  - `data-table-schema.ts`, `data-table-types.ts`, `data-table-utilities.ts`: Type-safe table definitions leveraging shared workspace types.

---

## Routing Structure (Implemented Scope)

### 1. The `(auth)` Sub-Tree

Handles user session instantiation.

- **`/sign-in`**: Login flows.
- **`/sign-up`**: Registration.

### 2. The `(dashboard)` Sub-Tree

The secure boundary wrapped in a global `layout.tsx`.

#### Deployed Feature Routes

- **`/` (Home / Dashboard Root)**
  - **Analytics UI**: Top-line metrics.
  - **P2P Notification Engine**: Handling group joins and settlements.
  - **OCR & Document Upload Pipeline**: Interacts with **`apps/api/upload`** and **`apps/api/ocr/process`**.
  - **Action Shims**: Global buttons for Transactions and Splits.

- **`/transactions`**
  - **Rich Data Grid**: LEveraging `@tanstack/react-table` with server-side integration.
  - **Native Export CSV Logic**: Browser-native blob generation.
  - **Contextual Form Editor**: Integration with the `TransactionViewer` panel.

- **`/subscriptions`**
  - **Pattern Recognition**: Consumes `GET /api/subscriptions/detect` powered by **`expent_core`** heuristics.

- **`/settings`**
  - **Budgets**: User-defined spending limits per category, visualized via the `BudgetHealthWidget`.
  - **User Controls**: Profile management and theme preferences.

---

## Data Fetching & Architecture Patterns

Expent frontend strictly follows explicit standardization to bridge reliably to the Rust **`apps/api`**:

1. **Edge Middleware (`src/proxy.ts`)**: Intercepts requests to check for `better-auth` session cookies and redirects to `/sign-in` if missing.
2. **API Routing**: Pages fetch from `NEXT_PUBLIC_API_BASE_URL` (defaults to `http://localhost:7878`) with `credentials: "include"`.
3. **Aggressive Optimistic Updating (React Query)**: Uses `useMutation` with cache invalidation for a hyper-responsive UI.
4. **Form Standardization**: Leveraging React 19 forms and isolated state management.
5. **Toast Notifications**: Consistent feedback via the `@expent/ui` `goey-toaster` component.
6. **Type Safety**: The dashboard uses types generated from the Rust backend via `ts-rs`, ensuring that models re-exported by **`expent_core`** are perfectly synced with the frontend.

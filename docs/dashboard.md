# Expent Dashboard Architecture & UI Documentation

This document covers the architectural layout, frontend stack, routing structure, and component methodology established for the Expent web dashboard. 

## Architectural Overview

The dashboard is built within the `apps/dashboard` monorepo package. It leverages cutting-edge React ecosystem tools tailored for highly dynamic, visually heavy reporting and transaction management interfaces.

*   **Framework**: Next.js 16 (Strictly utilizing the `App Router` under `src/app`).
*   **Core Library**: React 19.
*   **Compilation**: Turbocharged with the `babel-plugin-react-compiler` (React Compiler) for automatic memoization without explicit hook wrapping.
*   **State Management**: 
    - Server Cache / Action Fetches: `@tanstack/react-query`
    - Client Global App State: `zustand`
*   **Styling**: Tailwind CSS v4 & `next-themes` (for native Dark/Light mode layout switching).
*   **Animations**: `motion` (modern lightweight iteration for layout mounting & micro-interactions).
*   **Dependencies**: Imports strongly from shared workspace packages (`@expent/ui` for strict custom UI components and `@expent/types` for TypeScript bounds).
*   **Authentication**: Directly wired into `better-auth` using their native React bindings and Edge boundaries.

---

## Codebase Structure & Component Strategy

The codebase is logically split in the `src/` directory to handle complex React scaling cleanly by decoupling core page logic from deeply styled UI blocks.

### `src/components/`
- **`auth/`**: Contains the `AuthGuard` component — a React wrapper that checks session state and redirects unauthenticated users to `/sign-in`. Used in the `(dashboard)/layout.tsx`.
- **`layout/`**: Holds the global UI wiring. Contains exactly:
  - `app-sidebar.tsx`, `app-navbar.tsx`, `nav-main.tsx`, and `nav-user.tsx` defining the persistent navigation shells. Contains wrappers for responsive sidebar logic and mobile drawer adaptations.
  - `sidebar-wrapper.tsx` and `sidebar-client.tsx` for persistent sidebar state management.
  - `custom-sidebar-trigger.tsx` for mobile hamburger toggle.
- **`transactions/`**: Heavily targeted business logic forms:
  - `manual-transaction-dialog.tsx`: The form for injecting raw new financial actions.
  - `split-dialog.tsx`: Formulating fractional person-to-person P2P distribution of payments.
  - `transaction-viewer.tsx`: The detailed slide-out inspecting row-level transactions.
- **`data-table/` & `tool-ui/`**: Advanced headless table components driven natively by `@tanstack/react-table`, accompanied tightly by standalone widget shells like `approval-card`, `order-summary`, `progress-tracker`, and `preferences-panel`.

### `src/hooks/` & `src/lib/`
- **Hooks**: Houses `use-mobile.ts` (tailoring media queries into React scope) alongside state abstractions.
- **Libs Environment**: 
  - `query-client.ts`: Bootstraps global React Query logic, stale-times, and retry mechanisms.
  - `auth-client.ts`: Creates the `better-auth/react` client with `passkeyClient()` and `usernameClient()` plugins. Exports `signIn`, `signUp`, `useSession`, `signOut` hooks. Includes built-in `429` rate-limit interception logging `X-Retry-After` headers.
  - `data-table-schema.ts`, `data-table-types.ts`, `data-table-utilities.ts`: Strictly isolate type-safe table column definitions, formatting utilities, and schema bindings. These types often leverage or are derived from the shared `@expent/types` package to maintain end-to-end type safety with the backend API.

---

## Routing Structure (Implemented Scope)

The frontend cleanly divides layouts using Next.js **Route Groups** to explicitly isolate Authentication flows from the secure Dashboard environment.

### 1. The `(auth)` Sub-Tree
Handles user session instantiation locally without downloading the heavy dashboard chunk sizes.

*   **`/sign-in`**: Login flows handling Email/Password combinations + Passkeys via the `@better-auth/passkey` integration.
*   **`/sign-up`**: Onboarding screens registering identities bound to the central backend users database.

### 2. The `(dashboard)` Sub-Tree
The secure boundary. It is wrapped in a dedicated `layout.tsx` which enforces Auth Guards globally (redirecting unauthenticated requests smoothly to `/sign-in`), handles the hydration of the side-navigation shell, and parses ambient User contexts.

#### Deployed Feature Routes
*   **`/` (Home / Dashboard Root)** 
    - **Purpose**: The primary landing interface.
    - **Present Implementation**:
        - **Analytics UI**: Renders generic Top-line metrics (Total Balance/Expected Income) dynamically calculating across caches.
        - **P2P Notification Engine**: Iterates pending inbound P2P objects mapping outwards through the `ApprovalCard` triggering explicit UI mutations around "Join Group" or "Accept Settlement".
        - **OCR & Document Upload Pipeline**: Complete pipeline invoking `ProgressTracker` steps. Posts natively securely via `POST /api/upload` forwarding keys to `POST /api/process-image-ocr`. Populates parsed values into an interactive `OrderSummary` before explicitly dumping it into the Database tables.
        - **Action Shims**: Injects global top-level buttons for `ManualTransactionDialog` and `SplitDialog`.
    
*   **`/transactions`** 
    - **Purpose**: Fully functional headless data-grid mapped leveraging `@tanstack/react-table`.
    - **Present Implementation**:
        - **Rich Data Grid**: Complex row generation bounding Manual additions or Bank syncs. Has client-side memory searching, sorting by date, and grouping via Tabs (Income vs. Expense logic filters).
        - **Native Export CSV Logic**: Integrates Blob encodings creating downloadable `.csv` reports natively within client browsers without server cost.
        - **Contextual Form Editor**: Built-in dropdown row triggers embedding the robust `TransactionViewer` panel, aggressively tied against `useMutation` logic causing hyper-responsive updates on `PATCH` commands.

*   **`/subscriptions`** 
    - **Purpose**: Pattern recognition flagger handling automated expenses.
    - **Present Implementation**:
        - Aggressively leverages `useQuery` targeting the heuristic backend at `GET /api/subscriptions/detect`. It iterates outputs natively into card elements visualizing data like `sub.cycle`, tracking dates, and projected `amount` without blocking the standard DOM.

*   **`/settings`** 
    - **Purpose**: User controls hub.
    - **Present Implementation**:
        - Profile parsing mapped immediately to globally authenticated `useSession()` user constraints.
        - Integrates precisely utilizing `PreferencesPanel` which seamlessly intercepts `next-theme` states for application-wide Dark/Light/System bindings globally.

*   **`/contacts` & `/p2p` (Standalone Scaffolds)**
    - **Present Implementation**: Top level `/p2p` and `/contacts` pages themselves are scaffolding. Core functionality (P2P Acceptance, P2P Splitting, contact assignment) happens purely within widget environments natively embedded within the Root Dashboard (`/`) and `Transactions` table views directly.

---

## Data Fetching & Architecture Patterns

Expent frontend strictly follows explicit standardization to bridge reliably to the Rust `apps/server`:

1. **Edge Middleware (`src/proxy.ts`)**: Acts as next.js middleware intercepting every non-static request. Checks for Better Auth session cookies (`better-auth.session_token`, `better-auth.session-token`, `__Secure-better-auth.session_token`, `__Secure-better-auth.session-token`) and redirects unauthenticated users to `/sign-in`. Public routes (`/sign-in`, `/sign-up`) bypass this check.
2. **API Routing**: Pages fetch directly from `NEXT_PUBLIC_API_BASE_URL` (defaults to `http://localhost:7878`) with `credentials: "include"` to pass session cookies through to the Rust server.
3. **Aggressive Optimistic Updating (React Query)**:
   - Deeply interactive pages (Transactions, P2P Accept flow) avoid blocking standard HTTP loading cycles. 
   - They use `useMutation` with `onSuccess` cache invalidation via `queryClient.invalidateQueries`. When a user deletes a row or hits "Accept Request", the UI reflects this after server confirmation using toast notifications (`goey-toaster`).
4. **Form Standardization**: Complex payload-generating blocks avoid generic `onChange` prop drillings by leveraging native React 19 forms and isolated state management to prevent macro DOM layout repaints upon keystrokes.
5. **Toast Notifications**: All mutation outcomes (success/error) surface via the custom `@expent/ui` `goey-toaster` component for consistent user feedback.

# 🛠️ Expent Development Roadmap

> **Last audited**: 2026-04-04 — Full codebase review across `apps/server`, `apps/dashboard`, `apps/ocr`, `crates/*`, and `packages/*`.
> **Stack**: Rust (Axum/SeaORM) backend · Next.js 16 / React 19 dashboard · Python (FastAPI/Gemini) OCR · Cloudflare R2 storage · SQLite (dev) / PostgreSQL (prod)

---

## 📐 Development Conventions (Dress Code)

> These rules apply to **every task** in this roadmap. No exceptions.

### Code Splitting — No Monoliths

- **Dashboard (`apps/dashboard`)**: Business logic, API calls, and data transformations go in `/lib/` files (e.g., `lib/api-client.ts`, `lib/transaction-utils.ts`). Components import from `lib/`. Pages stay thin — they compose components and call hooks, they don't contain raw fetch logic or heavy processing.
- **Server (`apps/server`)**: Route handlers live in `src/routes/*.rs` modules. `main.rs` only sets up the app, state, and mounts routers. Each route module imports its logic from `crates/db` (SmartMerge and friends). No 600-line `main.rs` files.
- **DB Crate (`crates/db`)**: Complex multi-step operations stay in `SmartMerge` methods. If a method grows beyond ~50 lines, break it into helper functions in the same file or a dedicated submodule.
- **General rule**: If a file exceeds ~200 lines, it's time to split. Extract logical pieces into separate files and re-export from a `mod.rs` / `index.ts`.

### Commit Discipline

- **Commit after each completed task**: When a task (e.g., "1.4 — `GET /api/contacts` endpoint") is done and verified working, commit it immediately with a descriptive message like `feat(server): add GET /api/contacts endpoint`.
- **Don't batch unrelated changes**: One task = one commit. If a task naturally splits into backend + frontend, two commits are fine (`feat(server): ...` then `feat(dashboard): ...`).
- **Never leave the codebase broken**: If something is half-done at the end of a session, stash or revert. The `main` branch must always build and run.

### Commit Message Format
```
<type>(<scope>): <short description>

Types: feat, fix, refactor, chore, docs
Scopes: server, dashboard, db, ocr, ui, auth, migration
```

---

## 📊 Current State Audit — What Exists vs What's Missing

| Area | What Works Today ✅ | What's Missing / Broken ❌ |
|:---|:---|:---|
| **Auth** | Sign-in/Sign-up, Google OAuth, Passkey client configured, session management | No profile edit page, no passkey management UI, no 2FA toggle UI, no active sessions view, sign-out does raw redirect instead of `authClient.signOut()` |
| **Transactions** | List, manual create, OCR create, edit (via drawer), delete (soft), split, export CSV | No wallet/contact selection on manual create, no edit button in table row actions, categories hardcoded (not from DB), `notes` field not persisted, `source` shows raw enum |
| **OCR** | Upload → classify → Gemini extract → auto-create transaction end-to-end | Raw JSON still shown instead of a draft form, no auto-creation of `Contact`/`ContactIdentifier` from counterparty data, `purpose_tag` incorrectly set to `counterparty_name` |
| **Contacts** | Page route exists, sidebar link present | **Completely empty scaffold** — no API endpoints, no CRUD, no list, no search, no detail page |
| **Subscriptions** | Detection from transaction patterns, subscription cards displayed | No "Confirm" action persists to DB, no alert configuration, no charge history view |
| **P2P / Groups** | Create group, invite members, accept P2P, shared ledger page with group transactions | No `title`/`description` on `ledger_tabs` creation from UI, P2P root page is placeholder, `sender_user_id` shows raw UUID, no reject button |
| **Wallets** | `create_wallet` backend function exists, schema defined | **No API endpoint**, **no UI**, not linked to any transaction flow |
| **Reconciliation** | Schema tables (`bank_statement_rows`, `statement_txn_matches`) exist | **No API endpoints**, **no UI**, no upload/parse/match logic |
| **Settings** | Profile display (read-only), theme toggle, placeholder notification & security cards | No profile editing, no avatar upload, no passkey management, no active sessions |
| **Server** | All 30 entity models, migrations, indexes, IDOR protection | `main.rs` is a 600-line monolith — no route modules, no error middleware, no pagination, many missing endpoints |

---

## 🎯 Phase 0: Critical Fixes & Quick Wins

> **Goal**: Fix broken things and low-hanging fruit before adding features.

### Backend Fixes

- [ ] **0.1** Fix sign-out: Replace `window.location.href = "/sign-in"` in `settings/page.tsx` with `authClient.signOut()` then redirect
- [ ] **0.2** Fix OCR `purpose_tag` mapping: Currently sets `purpose_tag` to `counterparty_name` in `process_ocr()` (`crates/db/src/lib.rs:92`). Should set it to a proper category or `None` — `counterparty_name` belongs in `p2p_transfers` only
- [ ] **0.3** Fix GPay datetime parsing: `date: Set(Utc::now().into())` ignores `gpay.datetime_str` — should parse it when available (`crates/db/src/lib.rs:89`)
- [ ] **0.4** Fix P2P root page: `/p2p/page.tsx` shows "Hello" placeholder — redirect to `/p2p/pending` or show an overview
- [ ] **0.5** Add `notes` field to `transactions` or `transaction_metadata` — the drawer has a "Personal Note" input that's never persisted
- [ ] **0.6** Fix sender display in P2P cards: Shows raw `sender_user_id` UUID — resolve to sender name via backend join

### Frontend Fixes

- [ ] **0.7** Extract duplicate code: `updateMutation`, `deleteMutation`, API fetch patterns are copy-pasted across `page.tsx` (dashboard) and `transactions/page.tsx` — extract into shared hooks (`useTransactions`, `useP2P`)
- [ ] **0.8** Fix `ManualTransactionDialog`: `direction` state types use `any` cast — type properly
- [ ] **0.9** Add consistent loading/error states across all pages
- [ ] **0.10** Fix `TransactionViewer` — "Personal Note" input doesn't submit, Category selector is hardcoded

---

## 🔧 Phase 1: Server Architecture Refactor

> **Goal**: Split the monolithic `main.rs` into modular route handlers and add missing foundational endpoints.

### Server Modularization

- [ ] **1.1** Create route module structure:
  ```
  apps/server/src/
  ├── main.rs              (slim: app setup + state)
  ├── routes/
  │   ├── mod.rs
  │   ├── transactions.rs
  │   ├── contacts.rs
  │   ├── wallets.rs
  │   ├── subscriptions.rs
  │   ├── p2p.rs
  │   ├── groups.rs
  │   ├── uploads.rs
  │   ├── users.rs
  │   └── reconciliation.rs
  ├── middleware/
  │   ├── mod.rs
  │   └── error.rs         (unified ApiError type)
  └── extractors.rs
  ```
- [ ] **1.2** Create unified `ApiError` enum implementing `IntoResponse` — replace scattered `(StatusCode, String)` pattern
- [ ] **1.3** Add pagination to `list_transactions` — currently returns ALL user transactions with no limit/offset

### Missing API Endpoints

- [ ] **1.4** `GET /api/contacts` — List contacts for logged-in user (via `contact_links` join)
- [ ] **1.5** `POST /api/contacts` — Create contact + `contact_link` for user
- [ ] **1.6** `PUT /api/contacts/:id` — Update contact name, phone, `is_pinned`
- [ ] **1.7** `DELETE /api/contacts/:id` — Remove `contact_link`
- [ ] **1.8** `GET /api/contacts/:id` — Get contact detail with `contact_identifiers` and transaction history
- [ ] **1.9** `POST /api/contacts/:id/identifiers` — Add UPI/Phone/BankAcc identifier
- [ ] **1.10** `GET /api/wallets` — List user's wallets
- [ ] **1.11** `POST /api/wallets` — Create wallet (name, type, initial balance)
- [ ] **1.12** `PUT /api/wallets/:id` — Update wallet name/balance
- [ ] **1.13** `PUT /api/users/profile` — Update name, username, image
- [ ] **1.14** `GET /api/users/upi` — List user's UPI IDs
- [ ] **1.15** `POST /api/users/upi` — Add UPI ID
- [ ] **1.16** `PUT /api/users/upi/:id/make-primary` — Set primary UPI
- [ ] **1.17** `POST /api/p2p/reject` — Reject a P2P request
- [ ] **1.18** `GET /api/subscriptions` — List confirmed/persisted subscriptions
- [ ] **1.19** `POST /api/subscriptions` — Confirm a detected subscription (persist to DB)
- [ ] **1.20** `DELETE /api/subscriptions/:id` — Stop tracking
- [ ] **1.21** `POST /api/subscriptions/:id/alerts` — Configure `sub_alerts`
- [ ] **1.22** `GET /api/categories` — List user's custom categories
- [ ] **1.23** `POST /api/categories` — Create custom category
- [ ] **1.24** `DELETE /api/categories/:id` — Delete custom category

---

## 🚀 Phase 2: OCR-to-Form Transformation & Entity Auto-Creation

> **The Problem**: OCR currently dumps raw JSON, and it fails to link or create vendors/users automatically.

### OCR Review UI

- [ ] **2.1** Create `ReviewTransactionForm` component: Maps `GPayExtraction` / `GenericOCRResponse` fields into editable form inputs (amount, date, counterparty, UPI ID, direction)
- [ ] **2.2** Replace the current `OrderSummary` + raw JSON card in dashboard with `ReviewTransactionForm`
- [ ] **2.3** Allow user to edit mapped fields before "Confirm & Save" — currently the transaction is created on backend before user reviews
- [ ] **2.4** **Architecture change**: Split OCR into two steps:
  1. `POST /api/process-image-ocr` → returns structured OCR data (NOT a saved transaction)
  2. `POST /api/transactions/from-ocr` → user-confirmed data creates the transaction
- [ ] **2.5** Use `goey-toaster` to confirm when a new contact is "Auto-created from Receipt"

### Auto-Contact Creation ("Shadow Contact" Logic)

- [ ] **2.6** Update `SmartMerge::process_ocr()` GPay path to:
  1. Check if `counterparty_name` + `counterparty_upi_id` exists in `contacts` / `contact_identifiers`
  2. If not found → auto-create `Contact` + `ContactIdentifier` (UPI type) + `contact_link` for user
  3. Create `txn_parties` record linking the contact to the transaction
- [ ] **2.7** Store `r2_file_url` in `transaction_sources` — currently always set to `None` even though we have the R2 key

### 🛠️ Gemini CLI Prompt Block #1
> "Refactor the OCR acceptance flow in `apps/dashboard`. Instead of rendering the raw JSON from `POST /api/process-image-ocr`, create a `ReviewTransactionForm` component.
> - **Dynamic Mapping**: Automatically populate form fields from the `GPayExtraction` or `OCRResponse` objects (amount, date, counterparty).
> - **Entity Logic**: On the backend (`apps/server`), update the OCR processing handler to check if the `counterparty_name` exists in the `contacts` table. If it does not exist, create a new `Contact` and a `ContactIdentifier` (mapping the UPI ID or phone) before saving the `transaction` and linking them via `txn_parties`.
> - **UI Feedback**: Use the `@expent/ui` `goey-toaster` to confirm when a new contact is 'Auto-created from Receipt'."

---

## 💰 Phase 3: Transaction Intelligence & Manual Entry Overhaul

> **The Problem**: Manual entry lacks categories, payment methods (Wallets), and person selection.

### Manual Transaction Enhancement

- [ ] **3.1** Add `Wallet` selector to `ManualTransactionDialog`: Fetch from `GET /api/wallets`, map to `source_wallet_id` / `destination_wallet_id`
- [ ] **3.2** Add `Contact` picker to `ManualTransactionDialog`: Searchable combobox querying `GET /api/contacts`, with "Quick Add" button for new contacts
- [ ] **3.3** Update `CreateManualTransactionRequest` in server to accept optional `source_wallet_id`, `destination_wallet_id`, `contact_id`
- [ ] **3.4** Update `SmartMerge::create_transaction()` to persist wallet IDs and create `txn_parties` records

### Category Management

- [ ] **3.5** Create `categories` migration table (or dedicated handler for unique `purpose_tag` strings) — **decision needed: dedicated table vs freeform**
- [ ] **3.6** Replace hardcoded category list in `TransactionViewer` (`SelectContent` line 112-119) with dynamic data from `GET /api/categories`
- [ ] **3.7** Create `CategoriesPanel` component in `/settings` for adding/deleting custom tags via CRUD on the API

### Table UX

- [ ] **3.8** Add **Edit** action button to transaction table dropdown menu (currently only "Split" and "Delete") — opens `TransactionViewer` in mutable "Edit Mode"
- [ ] **3.9** Add "Quick Upload" drop zone at top of transactions page (currently only on dashboard)
- [ ] **3.10** Add `purpose_tag` as visible column in transactions table
- [ ] **3.11** Add faceted filters for `purpose_tag` and `source` type in transactions table toolbar

### 🛠️ Gemini CLI Prompt Block #2
> "Enhance the `ManualTransactionDialog` in `apps/dashboard/src/components/transactions/`:
> - **Wallet Selection**: Add a dropdown to select the `source_wallet_id` or `destination_wallet_id`. This must fetch the user's available wallets (Cash, Bank, UPI) from `GET /api/wallets`.
> - **Contact Picker**: Replace the text input for 'Paid To/From' with a searchable command-palette component that queries `/api/contacts`. Allow a 'Quick Add' button to create a new contact record if no match is found.
> - **Category Overhaul**: Add a category selector that pulls from the `purpose_tag` field in the `transactions` table.
> - **Transaction Table**: Add an 'Edit' button to each row in the `Transactions` table that opens the `TransactionViewer` in a mutable 'Edit Mode'."

---

## 📇 Phase 4: The "Contact & Vendor" Detail Engine

> **The Problem**: The Contacts page is just a scaffold and doesn't show transaction history or vendor-specific data.

### Contacts Page

- [ ] **4.1** Build contacts list page at `/contacts/page.tsx`:
  - Fetch from `GET /api/contacts`
  - Search/filter by name
  - Pin/unpin contacts
  - "Add Contact" dialog with name, phone, UPI ID fields
- [ ] **4.2** Create `ContactCard` component: name, identifiers as chips, pin status, transaction count

### Contact Detail Page

- [ ] **4.3** Create `/contacts/[id]/page.tsx` dynamic route:
  - Fetch contact details + identifiers from `GET /api/contacts/:id`
  - Use `is_merchant` flag from `p2p_transfers` to toggle between "Person" and "Vendor" view (Store icon vs User icon)
  - List all `contact_identifiers` as copyable chips (UPI logo, Phone icon, Bank icon per `IdentifierType`)
- [ ] **4.4** Add **Transaction History** section: filtered list from `GET /api/transactions?contact_id=...`
- [ ] **4.5** Add "Add Identifier" dialog to contact detail page
- [ ] **4.6** Show "On Expent ✓" badge if `contact_identifiers.linked_user_id` is set

### 🛠️ Gemini CLI Prompt Block #3
> "Build out the individual contact view at `apps/dashboard/src/app/(dashboard)/contacts/[id]/page.tsx`:
> - **Classification**: Use the `is_merchant` boolean from `p2p_transfers` to toggle the UI between a 'Person' view and a 'Vendor' view.
> - **Transaction History**: Implement a data table that fetches all transactions where this contact's `id` appears in `txn_parties`.
> - **Identifier List**: Display all `contact_identifiers` (UPI, Phone, Bank Acc) as copyable chips using the `IdentifierType` enum."

---

## 💳 Phase 5: Wallets UI & Integration

> **Goal**: Let users manage payment methods and link them to transactions.

- [ ] **5.1** Create `/wallets/page.tsx` or add "Wallets" section to Settings
- [ ] **5.2** Build `WalletCard` component: wallet name, type icon (Cash/Bank/Credit Card/UPI), running balance
- [ ] **5.3** Add "Create Wallet" dialog: name, type dropdown (`WalletType` enum), initial balance
- [ ] **5.4** Add Wallets nav item to sidebar
- [ ] **5.5** Wire wallet selection into `ManualTransactionDialog` (depends on 3.1)
- [ ] **5.6** Show linked wallet info in transaction detail drawer when `source_wallet_id` or `destination_wallet_id` is set

---

## ⚙️ Phase 6: Profile, Settings & Security

> **Goal**: Enable user autonomy — profile editing, passkeys, active sessions.

### Profile Management

- [ ] **6.1** Make profile card editable: Add input fields for `name`, `username`, connect to `PUT /api/users/profile`
- [ ] **6.2** Add avatar upload: File picker → `POST /api/upload` → update `users.image` via profile endpoint
- [ ] **6.3** Show `email_verified` status badge next to email with "Verify Now" if unverified

### Security

- [ ] **6.4** Add "Passkey Management" section using `authClient.passkey.addPasskey()` and list/remove existing passkeys
- [ ] **6.5** Add "Two-Factor Authentication" toggle using `authClient` 2FA methods
- [ ] **6.6** Add "Active Sessions" list: Show IP, user agent (parsed), expiry, with "Revoke" button
- [ ] **6.7** Fix sign-out to use `authClient.signOut()` (duplicated from 0.1 for tracking)

### Notification Preferences

- [ ] **6.8** Build notification preferences with `PreferencesPanel`: Toggle email/push, subscription alert channel
- [ ] **6.9** Connect to `sub_alerts` table for per-subscription alert configuration

### 🛠️ Gemini CLI Prompt Block #4
> "Implement the Category and Profile management suite:
> - **Category CRUD**: Create a `CategoriesPanel` in `/settings` that allows users to add/delete custom tags. This will require a new migration to create a `categories` table or a dedicated handler to manage unique `purpose_tag` strings.
> - **Profile Editor**: Build the `/settings/profile` page. Connect it to `PUT /api/users/profile` to update `name`, `username`, and `image`.
> - **Avatar Upload**: Wire the profile image upload to the `crates/upload` pipeline, ensuring it saves the file to R2 and updates the `users.image` column.
> - **Security**: Add a 'Passkey Management' section using the `better-auth` client's `passkeyClient()` to add/remove WebAuthn credentials."

---

## 🤝 Phase 7: P2P, Ledger Tabs & Group Enhancements

> **The Problem**: Shared ledgers are "blank" (no description), and P2P flows are incomplete.

### P2P Improvements

- [ ] **7.1** Add "Reject" button to P2P approval cards — needs `POST /api/p2p/reject` endpoint
- [ ] **7.2** Show sender **name** instead of UUID — backend join on `users.name` for `sender_user_id`
- [ ] **7.3** Fix P2P root page: Add overview with stats (total owed, total owing) or redirect to pending

### Ledger Tabs

- [ ] **7.4** Update `POST /api/p2p/create` and `ledger_tabs` to accept mandatory `title` and `description` — stop creating "blank" ledgers
- [ ] **7.5** Add "Create Ledger Tab" UI: Form with title, counterparty picker, tab type (Lent/Borrowed), target amount
- [ ] **7.6** Connect to `SmartMerge::create_ledger_tab()` (already exists in backend)
- [ ] **7.7** Show ledger tabs with status badges (Open/Partially Paid/Settled)
- [ ] **7.8** Add "Register Repayment" action — connect to `SmartMerge::register_repayment()` (already exists)

### Groups

- [ ] **7.9** Show group **members list** with role badges and avatars (currently "Members" button does nothing)
- [ ] **7.10** Add group member management: Remove member, change role (admin ↔ member)
- [ ] **7.11** Fix CSS/Z-index of "Share" icon in P2P notification engine
- [ ] **7.12** Ensure `counterparty_id` or `receiver_email` is always visible in `P2PRequest` cards

### 🛠️ Gemini CLI Prompt Block #5
> "Fix the P2P and Reconciliation logic:
> - **Ledger Context**: Update `ledger_tabs` and the `POST /api/p2p/create` request to include a mandatory `title` and `description` (e.g., 'Rent Split' or 'Dinner') so ledgers are never created blank.
> - **UI Visibility**: Fix the CSS/Z-index of the 'Share' icon and ensure the `counterparty_id` or `receiver_email` is always visible in the `P2PRequest` card.
> - **Bank Reconciliation**: Implement the 'Bank Statement' (BS) interface. Use the `bank_statement_rows` table to display a list of bank transactions and use the `statement_txn_matches` table to show 'Suggested Matches' with their `confidence` score. Allow users to 'Confirm' or 'Ignore' the match."

---

## 🔄 Phase 8: Subscription Engine

> **Goal**: Go from detection-only to full subscription lifecycle management.

- [ ] **8.1** Add "Confirm" action to subscription cards: Calls `POST /api/subscriptions` to persist
- [ ] **8.2** Add "Ignore" action: Dismiss detection (needs dismissal mechanism — metadata or cache)
- [ ] **8.3** Show confirmed subscriptions separately (two sections: "Tracked" vs "Detected")
- [ ] **8.4** Add subscription detail view: charge history from `subscription_charges`, alert config
- [ ] **8.5** Add `detection_keywords` editing for auto-match customization
- [ ] **8.6** Add alert configuration: "Remind me X days before via Email/Push" → `sub_alerts` table
- [ ] **8.7** Build subscription detection background job for periodic scanning

---

## 🏦 Phase 9: Bank Statement Reconciliation (The "BS" Tab)

> **Goal**: Implement the Bank Statement workflow from the schema.

- [ ] **9.1** Create `/reconciliation/page.tsx` route
- [ ] **9.2** Add sidebar nav item for "Reconciliation"
- [ ] **9.3** Backend: `POST /api/statements/upload` — accept CSV/PDF, parse into `bank_statement_rows`
- [ ] **9.4** Backend: Matching engine — match rows against `transactions` by amount + date + narration keywords → store in `statement_txn_matches` with confidence score
- [ ] **9.5** Backend: `GET /api/statements` — List uploaded statements
- [ ] **9.6** Backend: `GET /api/statements/:id/matches` — List matches with confidence scores
- [ ] **9.7** Backend: `POST /api/statements/matches/confirm` — User confirms a match
- [ ] **9.8** Build split-view UI: Bank row (left) ↔ Suggested transaction match (right), confidence badge, Confirm/Ignore buttons
- [ ] **9.9** Add CSV parser for common Indian bank statement formats (ICICI, HDFC, SBI)

---

## 🏗️ Phase 10: Infrastructure & Code Quality

> **Goal**: Improve DX, code quality, and production readiness.

### Code Quality

- [ ] **10.1** Extract shared hooks: `useTransactions()`, `useP2P()`, `useContacts()`, `useWallets()` into `/hooks/`
- [ ] **10.2** Create shared API client in `/lib/api-client.ts` — handles base URL, credentials, error parsing (replace raw `fetch()` everywhere)
- [ ] **10.3** Generate TypeScript types from Rust models via `ts-rs` in `packages/types` — stop using inline types and `any`
- [ ] **10.4** Clean up unused imports (Breadcrumb components imported but never used in several pages)

### Testing

- [ ] **10.5** Add API integration tests for server endpoints
- [ ] **10.6** Add E2E tests for critical flows: login → create transaction → split → accept
- [ ] **10.7** Add input validation on all API endpoints (negative amounts currently accepted)

### Production Readiness

- [ ] **10.8** Add rate limiting middleware to Rust server (protect OCR and auth endpoints)
- [ ] **10.9** Add per-route body size limits (currently global 10MB)
- [ ] **10.10** Add `GET /api/health` health check endpoint
- [ ] **10.11** Environment variable validation on startup (fail fast, not mid-request panic)
- [ ] **10.12** Configure database connection pooling (currently SeaORM defaults)
- [ ] **10.13** Make CORS allowed origins configurable via env var (hardcoded `localhost:3000` today)

---

## 📋 Database / API Alignment Matrix

| Feature | Database Table(s) | API Endpoint | Server | Dashboard |
|:---|:---|:---|:---|:---|
| **Transactions CRUD** | `transactions`, `transaction_edits` | `GET/POST/PATCH/DELETE /api/transactions` | ✅ | ✅ |
| **Transaction Parties** | `txn_parties` | Auto-created with transaction | ⚠️ Not used in manual | ❌ Not shown |
| **Transaction Metadata** | `transaction_metadata` | Auto-created with OCR | ✅ | ❌ Not displayed |
| **Transaction Sources** | `transaction_sources` | Auto-created | ⚠️ `r2_file_url` null | ❌ No "View Receipt" |
| **Payment Method** | `wallets` | `GET/POST /api/wallets` | ❌ No endpoint | ❌ No UI |
| **Entity Tracking** | `contacts`, `contact_links`, `contact_identifiers` | `GET/POST/PUT/DELETE /api/contacts` | ❌ No endpoint | ❌ Empty scaffold |
| **User UPI IDs** | `user_upi_ids` | `GET/POST/PUT /api/users/upi` | ❌ No endpoint | ❌ No UI |
| **OCR Result Form** | `transaction_sources` | `POST /api/process-image-ocr` | ✅ | ⚠️ Raw JSON |
| **Categories** | (TBD — `purpose_tag` freeform) | `GET/POST/DELETE /api/categories` | ❌ No endpoint | ❌ Hardcoded list |
| **Subscriptions** | `subscriptions`, `subscription_charges`, `sub_alerts` | `GET /api/subscriptions/detect` | ⚠️ Detect only | ⚠️ No persist |
| **P2P Requests** | `p2p_requests` | `POST /api/p2p/create`, `POST /api/p2p/accept` | ⚠️ No reject | ⚠️ Shows UUID |
| **P2P Transfers** | `p2p_transfers` | Auto-created with GPay OCR | ✅ | ❌ Not shown |
| **Groups** | `groups`, `user_groups` | `GET/POST /api/groups` | ✅ | ✅ |
| **Ledger Tabs** | `ledger_tabs` | SmartMerge function exists | ⚠️ No endpoint | ❌ No UI |
| **Purchases / Items** | `purchases`, `purchase_items` | Auto-created with OCR | ✅ | ❌ Not displayed |
| **Reconciliation** | `bank_statement_rows`, `statement_txn_matches` | `GET/POST /api/statements` | ❌ No endpoint | ❌ No UI |
| **Passkeys** | `accounts` (provider: 'passkey') | `/api/auth/*` (better-auth) | ✅ Auth layer | ❌ No settings UI |
| **Profile Edit** | `users` | `PUT /api/users/profile` | ❌ No endpoint | ❌ Read-only |
| **Active Sessions** | `sessions` | `GET /api/auth/sessions` | ❌ No endpoint | ❌ No UI |

---

## 🚀 Execution Priority Order

### Tier 1 — Fix First (Blockers & Broken Things)
1. Sign-out fix (0.1)
2. OCR `purpose_tag` mapping fix (0.2)
3. P2P root page fix (0.4)
4. Sender name display fix (0.6)
5. Extract shared hooks & API client (0.7, 10.1, 10.2)

### Tier 2 — Foundation (Required for all later features)
6. Server modularization (1.1, 1.2)
7. Contact API endpoints (1.4–1.9)
8. Wallet API endpoints (1.10–1.12)
9. Profile API endpoint (1.13)
10. Categories API endpoints (1.22–1.24)
11. Pagination support (1.3)

### Tier 3 — Core Features (The meat)
12. OCR review form + two-step flow (2.1–2.5)
13. Auto-contact creation from OCR (2.6–2.7)
14. Manual transaction enhancement — wallets + contacts + categories (3.1–3.7)
15. Contacts page + detail page (4.1–4.6)
16. Transaction table UX: edit button, filters, quick upload (3.8–3.11)

### Tier 4 — Complete the Experience
17. Wallets UI (5.1–5.6)
18. Profile editing + avatar upload (6.1–6.3)
19. Security settings — passkeys, 2FA, sessions (6.4–6.7)
20. P2P & ledger enhancements (7.1–7.12)
21. Subscription lifecycle (8.1–8.7)

### Tier 5 — Advanced Features & Polish
22. Bank statement reconciliation (9.1–9.9)
23. Notification preferences (6.8–6.9)
24. Testing & validation (10.5–10.7)
25. Production hardening (10.8–10.13)

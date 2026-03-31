### Part 1: Architecture & Monorepo Mapping

Here is how your codebase maps to the system's runtime architecture:

- **`apps/web` (TanStack Start + React):** The frontend UI. It talks to your backend services and handles direct uploads to Cloudflare R2 using presigned URLs. You are already utilizing your custom `packages/ui` heavily here.
- **`apps/auth` (Rust):** Your dedicated authentication microservice (using `better-auth.rs`). It issues tokens and manages the user session lifecycle.
- **`apps/backend` (Hono + TypeScript):** The API Gateway and "Backend for Frontend" (BFF). It routes requests, orchestrates business logic, and consumes the type-safe definitions from `packages/types`.
- **`crates/db` (Rust + SeaORM):** The core database engine. This holds all your entity definitions, migration logic, and complex transaction queries. It exports types to TS using `ts-rs`.
- **Python OCR/PDF Worker (Implied by `pyproject.toml`):** A lightweight Python service (e.g., FastAPI) that listens for processing jobs. It grabs the image/PDF from R2, runs EasyOCR or `pdfplumber`, and sends structured JSON back to the backend.
- **`packages/types`:** The single source of truth for types. `crates/db` generates the `.ts` files here, which are then consumed by `apps/web` and `apps/backend`.

---

### Part 2: The Final Database Schema (SeaORM Entities)

Based on the highly optimized relational diagram you provided, here are the exact entities you will generate in `crates/db/src/entities/`:

**1. Identity & Auth**

- `users`: `id`, `email`, `name`, `phone`
- `user_auth`: `provider`, `token`, `refresh`
- `user_upi_ids`: `upi_id`, `is_primary`, `label`

**2. People / Contacts**

- `contacts`: `name`, `phone`, `is_pinned`
- `contact_identifiers`: `type`, `value`, `linked_user_id` (The deduplication engine)
- `contact_links`: `user` ↔ `contact` (Many-to-many relationship)

**3. Transactions**

- `transactions`: `amount`, `direction`, `date`, `source`, `status`, `purpose`
- `transaction_metadata`: `upi_txn_id`, `app_txn_id`, `app_name`, `contact_number` (Keeps the core table clean)
- `txn_parties`: `role`, `contact_id`, `user_id`
- `txn_links`: `peer_link_status` (For the P2P social syncing)
- `ocr_imports`: `image_url` (R2 link), `raw_text`, `status`
- `statement_imports`: `pdf_url` (R2 link), `bank`, `period`

**4. Bank Statements**

- `bank_statement_rows`: `date`, `desc`, `debit`, `credit`, `bal` (Raw parsed data)
- `statement_txn_matches`: `row_id` ↔ `txn_id`, `confidence` (The smart-merge resolution table)

**5. Purchases & Subscriptions**

- `purchases`: `vendor`, `total`, `order_id`
- `purchase_items`: `name`, `qty`, `price`, `sku` (For Amazon-style itemized receipts)
- `purchase_imports`: `pdf_url`, `vendor`, `raw`
- `subscriptions`: `name`, `amount`, `cycle`, `start_date`, `next_charge`
- `subscription_charges`: `charged_on`, `txn_id`, `status`, `amount`
- `sub_alerts`: `days_before`, `sent_at`, `channel`

---

### Part 3: Step-by-Step Execution Plan

Given your current folder structure, here is the exact path to build this out systematically without getting overwhelmed.

#### Phase 1: Database & Type Generation (The Foundation)

1.  **Define SeaORM Entities:** In `crates/db/src/entities/`, translate the schema above into Rust structs using the SeaORM macros.
2.  **Add `ts-rs`:** Annotate your Rust structs with `#[derive(TS)]` and set the export path to `packages/types/src/db/generated/`.
3.  **Run Migrations:** Apply the schema to NeonDB.
4.  **Export:** Run `cargo test` (or your build script) to generate the TypeScript types. Ensure `packages/types/src/index.ts` cleanly exports them.

#### Phase 2: Auth & Web Scaffold

1.  **Auth Service:** Finish `apps/auth/src/main.rs`. Ensure it correctly writes users to the NeonDB `users` table upon signup and issues secure session tokens.
2.  **TanStack Setup:** In `apps/web/src/routes/sign-in.tsx` and `sign-up.tsx`, hook up your UI components to talk to the auth service.
3.  **Hono API Gateway:** Set up `apps/backend/src/index.ts` to validate the auth tokens on incoming requests before hitting the database.

#### Phase 3: The Data Ingestion Engine (R2 + Python)

1.  **R2 Integration:** In your Hono backend (`apps/backend`), write a route that generates a Cloudflare R2 Presigned URL.
2.  **Web Upload:** Update `apps/web` so users can upload screenshots directly to that Presigned URL.
3.  **Python Worker:** Create a new folder (e.g., `apps/ocr-worker` or map it in `pyproject.toml`). Write the FastAPI/Python scripts using `EasyOCR` and `pdfplumber`.
4.  **Worker Endpoint:** The worker should take an R2 URL, download the file into memory, parse it, and return a JSON payload matching the `transaction_metadata` / `bank_statement_rows` types.

#### Phase 4: The Core Rust Logic (Deduplication & P2P)

1.  **Smart Merge Logic:** In `crates/db/src/lib.rs`, write the Rust function that takes the JSON payload from Python. It should query `transactions`, check for matching amounts/dates, check `transaction_metadata` for matching UPI IDs, and populate `statement_txn_matches`.
2.  **Contact Resolution:** Write the logic to fuzzy-match incoming `contact_identifiers` against the user's existing list to prevent duplicates.
3.  **P2P Handshake:** Build the APIs for sending a request (`txn_links` status: pending) and accepting it (mirroring the transaction in the receiver's ledger).

#### Phase 5: Dashboard & UI Polish

1.  **Data Fetching:** Use TanStack Router's loaders combined with React Query (in `apps/web/src/lib/query-client.ts`) to fetch ledger data from the Hono API.
2.  **Visuals:** Build out the Dashboards in `apps/web/src/routes/dashboard/index.tsx` using your custom shadcn UI components (Charts, Tables, Data Grids).
3.  **Manual Resolution UI:** Create the UI where users can view `statement_txn_matches` that have low confidence scores and manually approve or reject the merge.

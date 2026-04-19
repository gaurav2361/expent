System Role & Context You are Jules, an expert SDET and AI code assistant. You are generating the test suite for "Expent," a high-performance, monorepo-based expense tracking application.

Technical Stack & Environment Constraints Before generating any code, strictly adhere to the following architectural guidelines for the Expent monorepo:

Backend (crates/expent_core): Built with Rust and the Axum framework.
Frontend (apps/dashboard): Built with Next.js.
Testing Frameworks: \* Strictly use rstest for all Rust backend logic, data integrity checks, and unit tests.
Strictly use vitest for all frontend custom hooks, state transitions, and headless TypeScript utility functions.
Toolchain: The project utilizes a Nix-based declarative environment, relying on bacon for Rust test auto-reloading and bun/pnpm for Next.js. Ensure the generated tests are clean, idiomatic, and optimized for fast CI/CD execution without flaky timeouts.
Task Directive Review the provided testing specification markdown below. Your objective is to translate this specification into concrete, production-ready test implementations.

Execution Requirements:

Rust Backend Suites (crates/expent_core): _ Generate rstest test cases with appropriate fixtures for transaction creation, updates, and soft-deletes.
Implement mock database driver setups specifically to assert the Query Count Constraints (ensuring no $1 + N$ query regressions occur in list_transactions).
Include specific boundary tests for the Subscription Detection Heuristics (handling exact intervals vs. fuzzy date drift).
Next.js Frontend Suites (apps/dashboard): _ Write vitest files to validate the useTransactions hook resilience.
Include mocks for the React Query queryClient to validate cache invalidation (queryKey: ["transactions"]), error handling (mocking 500 API responses), and loading skeleton triggers.
Data Integrity: \* Draft the "Double-Entry" Wallet-Ledger Parity check as a robust Rust integration test.
Output Format: Provide the exact file paths as headers (e.g., crates/expent_core/src/services/transaction_tests.rs) followed by the corresponding code blocks.
Specification:

1. Backend Service Logic (crates/expent_core)
   1.1 Transaction Creation (create_transaction)
   Logic: Uses a database transaction to ensure atomicity between creating the record and updating wallet balances.
   Tests:
   Success (Expense): Input amount: 100, direction: OUT, source_wallet_id.
   Outcome: Transaction record created; Wallet balance reduced by 100.
   Success (Income): Input amount: 500, direction: IN, destination_wallet_id.
   Outcome: Transaction record created; Wallet balance increased by 500.
   Validation: Input amount: -10 or 0.
   Outcome: Error ValidationError (Amount must be positive).
   Contact Linking: Input contact_id.
   Outcome: Entry created in txn_parties table with role COUNTERPARTY.
   1.2 Transaction Updates (update_transaction)
   Logic: Calculates the delta between the old amount and the new amount. If the wallet ID changes, it must revert the old wallet and apply the full amount to the new one.
   Tests:
   Amount Delta: Change amount from 100 to 120 (Expense).
   Outcome: Wallet balance reduced by an additional 20 (delta: -20).
   Direction Flip: Change from OUT 100 to IN 100.
   Outcome: Wallet balance corrected by +200 (+100 to reverse expense, +100 for income).
   Wallet Swap: Change source_wallet_id from Wallet A to Wallet B (Amount: 100).
   Outcome: Wallet A increased by 100 (reversal); Wallet B reduced by 100.
   Audit Logging: Every update must trigger a row in transaction_edits.
   1.3 Transaction Deletion (delete_transaction)
   Logic: Soft-delete via deleted_at timestamp. Must reverse the wallet impact completely.
   Tests:
   Reversal: Delete an OUT transaction of 50.
   Outcome: Wallet balance increased by 50; deleted_at is set.
   Idempotency: Attempt to delete an already deleted transaction.
   Outcome: No further wallet changes; Error or graceful skip.
   1.4 Optimized Listing (list_transactions)
   Logic: Single query with joins for categories and batch fetching for wallet/contact names.
   Tests:
   Pagination Boundaries: Seed 15 items. Fetch limit: 10, offset: 10.
   Outcome: Returns exactly 5 items; total_count remains 15.
   Join Accuracy: Ensure category_name, source_wallet_name, and contact_name are not null for transactions that have those IDs linked.
   Security: Fetch transactions for User A using User B's session.
   Outcome: Returns 0 items or Unauthorized.
   1.5 Dashboard Analytics (get_dashboard_summary)
   Logic: Complex aggregation using SQL SUM, COUNT, and GROUP BY.
   Tests:
   Total Balance: Create 3 wallets with 100, 200, and -50 balances.
   Outcome: total_balance is 250.
   Monthly Spend Boundary: Create transaction on the last day of last month and first day of this month.
   Outcome: Only this month's transaction is included in monthly_spend.
   Weekly Trend Bucketing: Verify that transactions are correctly grouped by day name (e.g., "Mon", "Tue").
   Top Expenses: Verify that the list is sorted descending by total amount and correctly maps contact_id to name.
   1.6 Subscription Detection Heuristics
   Logic: Scans history for repetitive patterns.
   Tests:
   Detection: 3 transactions of ₹199 from "Netflix" exactly 1 month apart.
   Outcome: Returned as a detected subscription with cycle: MONTHLY.
   Fuzzy Matching: Transactions on the 1st, 2nd, and 1st of consecutive months.
   Outcome: Detected despite minor date drift.
2. Frontend State & Hook Testing (apps/dashboard)
   2.1 Hook Resilience (useTransactions)
   Tests:
   Loading States: Verify isTxnsLoading is true during fetch and DashboardSkeleton is rendered.
   Error Handling: Mock a 500 API error. Verify toast.error is triggered.
   Cache Invalidation: After deleteMutation.success, verify that queryKey: ["transactions"] is invalidated and a refetch occurs.
   2.2 Navigation View Transitions
   Tests:
   Prefetch Trigger: Verify queryClient.prefetchQuery is called when a sidebar item is hovered.
   Directional Logic: Manual check that clicking "Transactions" (forward) vs clicking "Logo" (back) triggers the correct CSS animations (nav-forward vs nav-back).
3. Data Integrity & Sync
   3.1 Wallet-Ledger Parity
   The "Double-Entry" Check: Periodically (via test script) calculate the sum of all non-deleted transactions for a wallet and compare it against the wallets.balance field.
   Requirement: They must be exactly equal.
4. Performance Benchmarks (Regression Testing)
   Query Count Constraint: Using a mock database driver, assert that list_transactions executes:
   1 Query for COUNT(\*).
   1 Query for Transaction list + Category join.
   1-2 Queries for Wallet/Contact batching.
   Failure: If it executes $1 + N$ queries, the test fails.
5. Implementation Roadmap
   Stage 1: expent_core Unit Tests (Focus on update_transaction and summary).
   Stage 2: Integration Tests (API Routes to Core Services).
   Stage 3: Vitest Setup for Dashboard Hooks.
   Stage 4: Continuous Integration (Github Actions) for all suites.

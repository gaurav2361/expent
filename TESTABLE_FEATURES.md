# Testable Features and Functions

This document lists the core functions of the Expent ecosystem, their descriptions, and expected outcomes to serve as a reference for creating comprehensive test suites.

## 1. OCR Processing (Python FastAPI - `apps/ocr`)

| Function             | Description                                                                                                 | Expected Outcome                                                                         |
| -------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `classify_image`     | Uses Gemini to identify if a document is a Google Pay (`GPAY`) screenshot or a generic receipt (`GENERIC`). | Returns `"GPAY"` or `"GENERIC"` string.                                                  |
| `extract_from_bytes` | Orchestrates EasyOCR (for text context) and Gemini (for structured extraction) from raw file bytes.         | Returns a JSON object matching the requested schema (Amount, Date, Vendor, Items, etc.). |

---

## 2. Transaction Management (Rust Core - `crates/expent_core`)

| Function             | Description                                                                                               | Expected Outcome                                                                                                         |
| -------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `create_transaction` | Atomically creates a transaction in the DB, links it to a contact, and updates wallet balances.           | New transaction record exists; Wallet balances adjusted (Income adds, Expense subtracts).                                |
| `update_transaction` | Updates transaction details, creates an audit log in `transaction_edits`, and recalculates wallet deltas. | Fields updated; New entry in `transaction_edits`; Wallet balance corrected by the difference between old and new amount. |
| `delete_transaction` | Soft-deletes a transaction by setting `deleted_at` and reverses its impact on wallet balances.            | Transaction marked as deleted; Wallet balance restored to original state before this transaction.                        |
| `split_transaction`  | Splitting a transaction among participants.                                                               | Transaction updated with split details; P2P requests generated for other participants.                                   |

---

## 3. Subscription Detection (Rust Core - `crates/expent_core`)

| Function               | Description                                                                                          | Expected Outcome                                                                           |
| ---------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `detect_subscriptions` | Heuristic algorithm that scans transaction history for recurring patterns (Weekly, Monthly, Yearly). | Returns a list of potential subscriptions with frequency and predicted `next_charge_date`. |

---

## 4. Wallet & Ledger (Rust Core - `crates/expent_core`)

| Function                     | Description                                                                                                            | Expected Outcome                                                                                     |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `adjust_transaction_wallets` | Internal utility that calculates the delta between two transaction states and applies it to the corresponding wallets. | Wallets' `current_balance` is always in sync with the ledger, even after complex edits or deletions. |

---

## 5. Core Orchestration (Rust Core - `crates/expent_core`)

| Function     | Description                                                                                     | Expected Outcome                                                              |
| ------------ | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `Core::init` | Initializes Database (SeaORM), Auth (BetterAuth), Storage (S3/R2), and OCR Service connections. | A `Core` instance ready for service; System default categories ensured in DB. |

---

## 6. Categories (Rust Core - `crates/expent_core`)

| Function                   | Description                                                                          | Expected Outcome                                                                |
| -------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------- |
| `ensure_system_categories` | Seeds the database with a default set of transaction categories if they don't exist. | Database contains standard categories (e.g., Food, Transport, Rent) on startup. |

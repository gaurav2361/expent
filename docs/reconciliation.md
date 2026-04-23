# Bank Statement Reconciliation (`crates/reconciliation`)

The Reconciliation service allows users to upload raw bank statements and match them against recorded transactions, ensuring the digital ledger matches real-world bank records.

## 1. Logic Workflow

Reconciliation follows a three-step pipeline: **Ingestion**, **Fuzzy Matching**, and **Confirmation**.

### Step 1: Ingestion
Raw statement data is uploaded and parsed into `bank_statement_rows`. Each row represents a single line in a bank statement, containing a date, description, and debit/credit amount.

### Step 2: Fuzzy Matching
The `get_row_matches` engine attempts to find existing `transactions` that correspond to a statement row using a weighted scoring system:

| Factor | Weight/Logic |
|--------|--------------|
| **Amount** | Must be an exact match (absolute value). |
| **Date Range** | Within +/- 3 days of the statement date. |
| **Description** | Weighted match if the transaction `purpose_tag` appears in the statement narration. |
| **Base Score** | Starts at 70 if amount and date range match. |

### Step 3: Confirmation
When a user confirms a match, the system:
1.  Creates a record in `statement_txn_matches`.
2.  Flags the `bank_statement_row` as `is_matched = true`.
3.  Calculates the "Confidence Level" for audit reporting.

---

## 2. Service Hub Orchestration

Managed by **`expent_core::services::reconciliation`**.

- **Transaction Integrity**: Confirmation logic is wrapped in a database transaction. If the match fails to record, the row status remains unmatched.
- **Deduplication**: Unmatched rows are surfaced on the dashboard to prompt the user to either "Match" or "Create New Transaction" from the row data.

---

## 3. Database Schema

### `bank_statement_rows`
- Stores raw line items from CSV/PDF exports.
- `is_matched`: Boolean flag used to filter the dashboard "Inbox".

### `statement_txn_matches`
- The join table linking statement rows to finalized transactions.
- Includes `confidence` (Decimal) and `matched_at` (Timestamp) for audit trails.

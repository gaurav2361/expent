### Phase 1: Solving the "Cash" Problem (The Wallet Abstraction)

Currently, your system likely assumes money just "moves" between people. To introduce cash, we need to introduce **where** the money is sitting.

**The Solution: Accounts / Wallets Table**
We need an `accounts` (or `wallets`) table linked to the `user`.

- **Columns:** `id`, `user_id`, `name` (e.g., "HDFC Bank", "Left Pocket Cash"), `type` (Enum: `CASH`, `BANK`, `CREDIT_CARD`, `UPI_WALLET`), `balance`.
- **Transaction Update:** The `transaction` table must be updated to include `source_account_id` and `destination_account_id`.

**How it works:**

- If you get cash from an ATM, that's a transfer from your `BANK` account to your `CASH` account.
- If you pay someone ₹500 in cash for lunch, the transaction's `source_account_id` is your `CASH` account.
- This instantly gives you the ability to filter your dashboard by "Cash on hand" vs. "Money in the bank."

---

### Phase 2: Solving Loans & Partial Repayments (The "Tab" System)

You mentioned needing a "group" to link a loan of ₹5000 to a partial repayment of ₹2000. In financial systems, this is often called an "Invoice", "Tab", or "Settlement Thread".

**The Solution: The `ledger_tab` (or `loan_group`) Entity**
Create a new SeaORM entity dedicated strictly to tracking an overarching debt or loan.

- **Columns:** \* `id`
  - `creator_id` (You)
  - `counterparty_id` (The person you are lending to / borrowing from)
  - `tab_type` (Enum: `LENT`, `BORROWED`)
  - `title` (e.g., "March Rent Loan", "Vegas Trip Advance")
  - `target_amount` (e.g., ₹5000)
  - `status` (Enum: `OPEN`, `PARTIALLY_PAID`, `SETTLED`)

**Linking to Transactions:**
Update your `transaction` table to include a nullable foreign key: `ledger_tab_id`.

**How it works in practice:**

1.  **The Loan:** You lend your friend ₹5000. Expent creates a `ledger_tab` for ₹5000. Simultaneously, it creates a ₹5000 `transaction` (source: your bank, destination: them) and tags it with this `ledger_tab_id`.
2.  **The Partial Repayment:** A week later, they UPI you ₹2000. You log an incoming transaction and tag it with the exact same `ledger_tab_id`.
3.  **The Math:** To find out how much is still owed, Expent simply queries: `target_amount - SUM(transactions where ledger_tab_id = X)`. The current outstanding balance is always dynamically calculated, never manually guessed.

---

### Phase 3: Upgrading `SmartMerge` for the New Logic

Your `SmartMerge` namespace will be the heavy lifter here to ensure these new multi-step processes don't break.

**New `SmartMerge` Workflows:**

1.  **`SmartMerge::register_repayment(tab_id, amount, account_type)`**
    - _Begin DB Transaction._
    - Insert the new partial payment into `transaction`.
    - Query the total sum of transactions linked to `tab_id`.
    - If `SUM(repayments) >= target_amount`, automatically update the `ledger_tab` status to `SETTLED`.
    - Generate a notification/receipt for the counterparty.
    2.  **`SmartMerge::revert_transaction(transaction_id)`**
    - _Commit (or Rollback on error)._
    - If a user deletes a repayment, `SmartMerge` must check if that transaction belonged to a `ledger_tab`.
    - If it did, it must recalculate the tab's sum and revert the status from `SETTLED` back to `PARTIALLY_PAID` if necessary.

---

### Phase 4: Additional Database Schema Improvements

Since you are open to broader improvements for Expent's database layout, consider adding these guardrails:

**1. Soft Deletes (`deleted_at` timestamp)**
In financial databases, you should almost _never_ run a hard `DELETE` SQL command. If a user deletes a transaction, simply update a `deleted_at` column with the current timestamp. Update all your SeaORM read queries to filter `WHERE deleted_at IS NULL`. This preserves the paper trail if a user ever claims a transaction "disappeared" or if you need to restore data.

**2. Audit Logging for Edits**
You mentioned a user might change a dinner bill from ₹1000 to ₹1200. Add a simple `transaction_edits` table (`id`, `transaction_id`, `old_amount`, `new_amount`, `edited_at`). When `SmartMerge` handles an update cascade, have it drop a record here. It prevents arguments between friends about who changed the amount and when.

\*_3. "System" Users for External Cash_

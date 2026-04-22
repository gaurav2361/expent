# Smart Budgeting System Plan

## Objective
Implement a core financial feature that allows users to set monthly spending limits per category and visualize their progress against these limits on the dashboard.

## Phase 1: Database Infrastructure
1.  **Migration**: Create a new table `budgets` with columns:
    *   `id` (String, primary key)
    *   `user_id` (String, foreign key to `users`)
    *   `category_id` (String, optional foreign key to `categories`)
    *   `amount` (Decimal)
    *   `period` (String enum: `MONTHLY`, `WEEKLY`, `YEARLY` - defaulting to `MONTHLY`)
    *   `created_at` (DateTime)
    *   `updated_at` (DateTime)
2.  **Entities**: Update SeaORM entities in `crates/db/src/entities` to include the new `budgets` entity and establish relationships.

## Phase 2: Backend Logic (`crates/budgets`)
1.  **Crate Creation**: Set up a new `budgets` crate.
2.  **CRUD Operations**: Implement functions to create, read, update, and delete budgets.
3.  **Health Calculation**: Implement a `get_budget_health` function that calculates the total spending for a given category in the current period and returns the percentage consumed.
4.  **Integration**: Ensure `crates/expent_core` can utilize the new `budgets` crate.
5.  **API Routes**: Create Axum endpoints in `apps/api` (e.g., `GET /api/budgets`, `POST /api/budgets`, `GET /api/budgets/health`).

## Phase 3: Frontend Integration
1.  **API Client**: Add budget-related types to `@expent/types` and implement fetching hooks (`useBudgets`) in the dashboard.
2.  **Settings Panel**: Add a "Budgets" tab in the Settings section to manage limits per category.
3.  **Dashboard Widget**: Create a "Budget Health" widget (e.g., a progress bar or burn-down chart) on the main overview page to visualize spending against limits.

## Validation
- Add unit tests in `crates/budgets/src/tests.rs`.
- Run formatting and linting across the stack.
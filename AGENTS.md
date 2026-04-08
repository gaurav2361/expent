# Agent Instructions

## Tech Stack & Structure

- **Frontend**: `apps/dashboard` (Next.js), `apps/app` (Expo)
- **Backend**: `apps/api` (Rust Axum), `apps/ocr` (Python FastAPI)
- **Central Hub**: `crates/expent_core` (Orchestrates DB, Auth, Upload, OCR)
- **Shared**: `packages/types` (Shared TS/Rust types), `packages/ui` (Shared UI)
- **Testing**: `rstest` (Rust Backend & API), `vitest` (Next.js Headless Logic). No UI E2E.

## Package Managers

- **JS/TS**: **pnpm**: `pnpm install`, `pnpm dev`
- **Rust**: **cargo**: `cargo check`, `cargo run -p api`
- **Python**: **uv**: `uv sync`, `uv run uvicorn main:app`

## File-Scoped Commands

| Task          | Command                                   |
| ------------- | ----------------------------------------- |
| Typecheck     | `pnpm tsc --noEmit path/to/file.ts`       |
| Lint (JS)     | `pnpm biome check path/to/file.ts`        |
| Lint (Rust)   | `cargo clippy --fix -p <crate> -- <file>` |
| Test (Rust)   | `cargo test -p expent_core --lib`         |
| Test (JS/TS)  | `pnpm vitest run path/to/file.test.ts`    |
| Test (Python) | `uv run pytest apps/ocr/<file>`           |

## Key Conventions

- **Core Logic**: Business rules in `crates/expent_core/src/services/`. Split into granular files.
- **Entry Point**: API routes delegate to `expent_core::services`.
- **Database**: Pure entities in `crates/db/src/entities/`. No logic here.
- **Transactions**: Atomic operations MUST use `db.transaction`. Always adjust wallet balances.
- **Dependency Management**: Common deps in root `Cargo.toml`. Use `workspace = true`.
- **UI**: Components in `packages/ui`. Follow shadcn/ui patterns.
- **Types**: TS types in `packages/types/src/db/` are generated via `ts-rs`.
- **TDD**: Red-Green-Refactor mandatory. Leverage `#[rstest]` fixtures and cases for Rust validation.

## Documentation

- `AGENTS.md`: Canonical agent-facing documentation. Keep under 80 lines.
- `GEMINI.md`: foundational mandates for Gemini CLI specifically.
- `docs/core.md`: Deep dive into the Centralized Hub Architecture.

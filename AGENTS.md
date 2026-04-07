# Agent Instructions

## Tech Stack & Structure

- **Frontend**: `apps/dashboard` (Next.js), `apps/app` (Expo), `apps/web` (Vite)
- **Backend**: `apps/server` (Rust Axum), `apps/ocr` (Python FastAPI)
- **Shared**: `packages/types` (Shared TS/Rust types), `packages/ui` (Shared shadcn components)
- **Logic**: `crates/db` (SeaORM), `crates/auth` (Better Auth), `crates/upload` (S3/R2)

## Package Managers

- **JS/TS**: Use **pnpm**: `pnpm install`, `pnpm dev`, `pnpm build`
- **Rust**: Use **cargo**: `cargo check`, `cargo build`, `cargo run --bin server`
- **Python**: Use **uv**: `uv sync`, `uv run uvicorn main:app`

## File-Scoped Commands

| Task          | Command                                   |
| ------------- | ----------------------------------------- |
| Typecheck     | `pnpm tsc --noEmit path/to/file.ts`       |
| Lint (JS)     | `pnpm biome check path/to/file.ts`        |
| Lint (Rust)   | `cargo clippy --fix -p <crate> -- <file>` |
| Test (Rust)   | `cargo test -p <crate> --lib <module>`    |
| Test (Python) | `uv run pytest apps/ocr/<file>`           |

## Key Conventions

- **Database**: Logic in `crates/db/src/services/`. Entry point is `SmartMerge` in `crates/db/src/lib.rs`.
- **Migrations**: Managed in `crates/migration/`. Use `sea-orm-cli migrate up`.
- **Transactions**: Atomic operations MUST use `db.transaction`. Always adjust wallet balances.
- **Auth**: Uses Better Auth via `crates/auth`. Check `.env.example` for required keys.
- **UI**: Components in `packages/ui`. Follow shadcn/ui patterns and `lucide-react-native` for app.
- **Types**: TS types in `packages/types/src/db/` are generated via `ts-rs` from Rust entities.
- **TDD**: This project follows Test-Driven Architecture. Write tests for every function. Verify behavior frequently with automated tests.

## Documentation

- `AGENTS.md`: Canonical agent-facing documentation. Keep under 80 lines.
- `GEMINI.md`: foundational mandates for Gemini CLI specifically.

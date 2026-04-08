# Expent Database Migrations Architecture (`crates/migration`)

This document strictly defines the operational procedures and history mapping handled intrinsically by the `crates/migration` package. 

Unlike standard ORM implementations, Expent explicitly abstracts execution of database versioning cleanly into its own unified crate (`crates/migration`) driven directly by SeaORM internals.

## Architectural Overview

*   **Logic Engine**: `sea_orm_migration`.
*   **Database Standard**: Structured implicitly for SQLite compatibility natively.
*   **Execution Runtime**: Can be run entirely standalone `cargo run --manifest-path crates/migration/Cargo.toml` or securely automated at Rust `apps/server` startup.

---

## 1. Migration Topology

Every migration represents a precise deterministic schema difference executed linearly. The internal `src/` directory defines them matching explicit timestamped bounds.

### Currently Implemented Migration Sets
*The following operations are already mapped perfectly to `db::entities::*` and `docs/database_schema.md`.*

| ID / Timecode | Migration Focus | Operation Execution |
|---------------|-----------------|---------------------|
| `m20220101_000001` | **Base Architecture** | Instantiate foundational entities: `users`, `sessions`, `accounts`, `verifications`, `transactions`, `contact_links`, `p2p_requests`, `purchase_items`, `subscriptions`. (Note: The `files` entity was part of an earlier design and its functionality has since been absorbed or replaced by the dedicated `crates/upload` service for external S3/R2 storage, and is no longer directly represented in `db::entities::*`.) |
| `m20260331_092335` | **Groups Extrusion** | Devolves the standalone `groups` and mapping `user_groups` bridging tables (formerly `group_members`). |
| `m20260331_181001` | **Better Auth Parity**| Injects standard OAuth missing fields (`image`, `name`) down to `users` tables satisfying strict traits. |
| `m20260331_185523` | **Contacts Linkage** | Upgrades `transactions` table pointing foreign constraints toward `contact_links`. |
| `m20260401_000001` | **Indexing Boost** | Compiles deep SQLite `INDEX` commands targeting heavy lookup columns natively smoothing `GROUP BY` ops. |
| `m20260403_000001` | **Financial Refactor** | Heavy alter drops mapping `amount` to string definitions for fixed decimal parsing avoiding float arithmetic bleeding. |

---

## 2. Bootstrapping Execution

Instead of depending on unreliable external tooling, Expent executes standard upgrades inherently within the architecture bounds:

### The `Migrator` Struct (`src/lib.rs`)
The `lib.rs` file defines a `Migrator` struct implementing `sea_orm_migration::MigratorTrait`. It registers all migration files in explicit chronological order within the `migrations()` method, ensuring deterministic linear execution.

### Production / Runtime Automation
1. Found in `apps/server/src/main.rs`.
2. Invokes `Migrator::up(&db, None).await` at boot.
3. Because changes are executed progressively in the same transaction loop, an application *never* boots connected to a drifted database state.

### Manual Execution (`crates/migration/src/main.rs`)
For CI/CD sanity checks or sandbox wipes, the internal `main.rs` drops into SeaORM CLI mode natively:
```bash
# Rollback the last migration
cargo run -p migration -- down -n 1

# Force flush the database entirely
cargo run -p migration -- fresh
```

---

## 3. Strict Rules for Future Migrations

To maintain mapping parity safely with `docs/database_schema.md`:
1. **Never mutate old migrations:** If a table logic fails, append a *new* `.rs` migration file resolving the difference instead of actively mutating historical `m2022...` files.
2. **Abstract Foreign Keys**: Define `ForeignKey` relationships explicitly utilizing `sea_query::ForeignKey::create()` so SQLite strict mode enforces `ON DELETE CASCADE`.
3. **Rust Type Alignment**: After applying an `.rs` migration block, the `crates/db/src/entities/` block *MUST* instantly be re-synced using `sea-orm-cli` (e.g., `sea-orm-cli generate entity -o crates/db/src/entities`) to map database schemas to actual Rust structs, thereby ensuring type safety and preventing runtime panic states due to schema mismatches.

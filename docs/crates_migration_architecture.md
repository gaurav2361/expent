# Migration Crate Architecture Documentation

This document covers the `migration` library crate located at `crates/migration`. It holds the lifecycle progression of the SQLite schema.

---

## 1. Overview & Purpose

When developers alter the `database_schema.md` to add new fields (like enabling feature XYZ), existing users cannot simply have their databases deleted. The `migration` crate maps explicit state changes in Rust logic, using `sea-orm-migration`, to programmatically shape the underlying SQL database dynamically upon startup or deployment.

---

## 2. Core Mechanisms

### Chronological Migration Files
The crate structure defines isolated files in chronological sorting (e.g., `m20220101_000001_create_table.rs`).
Every file holds two mandatory async functions:
- `up()`: Executes the forward creation of tables, enum sets, columns, and relations.
- `down()`: Reverses the `up()` exact changes safely (e.g., dropping the table or column).

### The Migration CLI
Because this exists as a separated crate, it compiles entirely into a standalone CLI executor mapping database migrations independently of the `apps/server` lifecycle.

---

## 3. Server Integration

While it can execute as a CLI tool, Expent's primary deployment pipeline generally embeds the `Migrator::up()` call into the `apps/server` bootstrap code. When the server powers on, it checks if the local SQLite DB matches the latest migration manifest, applies pending operations safely, and then opens inbound traffic.

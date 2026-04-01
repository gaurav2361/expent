# SeaORM Migration CLI

This crate handles database schema migrations using [SeaORM](https://www.sea-ql.org/SeaORM/).

## Setup

Ensure your `DATABASE_URL` is set in the environment or `.env` file.

### Local SQLite Setup
For a local SQLite database, use:
```sh
DATABASE_URL="sqlite://expent.db?mode=rwc"
```
The `?mode=rwc` flag tells SQLite to create the file if it doesn't exist.

## Running Migrations

Run these commands from the root directory using the `--package migration` flag, or from within this directory.

- **Apply all pending migrations (Initialize DB)**
  ```sh
  cargo run --package migration -- up
  ```
- **Generate a new migration file**
  ```sh
  cargo run --package migration -- generate MIGRATION_NAME
  ```
- **Rollback last applied migration**
  ```sh
  cargo run --package migration -- down
  ```
- **Check migration status**
  ```sh
  cargo run --package migration -- status
  ```
- **Reset database (Drop all tables and re-run migrations)**
  ```sh
  cargo run --package migration -- fresh
  ```

## Advanced Commands


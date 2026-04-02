# DB Crate Architecture Documentation

This document describes the inner workings of the `db` library crate located at `crates/db`. It bridges the application with the persistence layer.

---

## 1. Overview & Purpose

The `db` crate is the central nervous system for Expent's data. Rather than the `server` holding raw SQL operations everywhere, this crate provides strongly typed structures (`entities`) matching the database topology perfectly, alongside a `SmartMerge` namespace dedicated to handling compound database actions safely within transactional bounds.

Because Expent involves money tracking, partial database updates (e.g., deducting an amount but failing to insert a receipt record) are catastrophic. This crate prevents such logic failures.

---

## 2. Core Mechanisms

### SeaORM Entities
In the `entities/` module, the crate houses structural mappings for every table (e.g., `user`, `transaction`, `contact`). These are used deeply across the application to provide strictly typed CRUD statements, effectively preventing SQL Injection attacks and ensuring schema versioning compatibility.

### `SmartMerge`
A domain logic wrapper. When the web interface triggers an event, it rarely equates to inserting a single row.
- **Example (`split_transaction`)**: A single API call asks to split a ₹1000 bill. 
  - `SmartMerge` opens an isolated Database Transaction.
  - It creates the initial `transaction`.
  - It iterates over the target contacts, inserting distinct `txn_parties`.
  - It generates multiple `p2p_requests` notifying the corresponding users.
  - *If any step errors, the entire procedure is rolled back.*

---

## 3. Cross-Crate Synergy

`crates/db` acts as the dependency root for many other features:
- `auth` consumes it to fetch `better-auth` credentials.
- `server` relies on its schemas to parse JSON responses easily (`Json<db::entities::transaction::Model>`).

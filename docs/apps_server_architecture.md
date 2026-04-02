# Server App Architecture Documentation

This document explores the foundational API Gateway execution application located at `apps/server`. Developed in Rust using the Axum framework, it represents the primary ingress point for the Expent web frontend.

---

## 1. Overview & Purpose

The `server` acts as the unified orchestrator. Unlike raw logic crates, this is a compilation target meant to run continuously as a daemon/server. It ties together authentication, database connections, networking, and OCR proxies into a cohesive JSON REST API structure properly configured with CORS protocols for secure web frontend integrations.

---

## 2. Core Architecture

### Context Management (`AppState`)
The application is entirely stateless between requests. However, structural dependencies like DB pools are injected via Axum Extractors.
```rust
struct AppState {
    db: DatabaseConnection,                         // Shared connection pool
    auth: Arc<better_auth::BetterAuth<SqliteAdapter>>, // Active Auth configuration
    upload_client: UploadClient,                    // AWS S3 / R2 Wrapper Config
    ocr_service: Arc<OcrService>,                   // Pointer to the Python API
}
```
This paradigm ensures thread-safe, incredibly fast asynchronous data processing securely isolated within `tokio` threads.

---

## 3. The Router API Space

The HTTP API is horizontally separated by logical intent.

### Transactions & P2P Router (`api_router`)
Located beneath `/api`, all handlers demand authorized extraction context (`session: AuthSession`).
- **REST Transactions**: Manages CRUD endpoints resolving to `SmartMerge` updates. E.g., `PATCH /transactions/:id`.
- **Peer-to-Peer Actions**: Facilitates `p2p_requests` creation, viewing (`/p2p/pending`), or execution flows (`/p2p/accept`). It bridges the gap between contacts splitting bills.

### Better Auth Integration
The router automatically absorbs `better-auth` handlers mapped to `$BASE_URL/api/auth`.
- Features like Magic Links, Password Hashes, Login/Logout, and OAuth redirects operate automatically here relying cleanly on `crates/auth`.

---

## 4. File Proxying & Cross-Container Workflows

Because `server` controls access privileges, external systems cannot be queried directly cleanly by the UI.

- **Storage Handling**: Uploads sent to `POST /upload` are instantly converted into `Bytes` payload, wrapped by `crates/upload`, resulting in a safe URL response.
- **OCR Pipelining**: When users hit `POST /process-image-ocr`, the server checks their session limits, validates file boundaries, and internally requests the Python `apps/ocr` worker over the local/VPC network without ever exposing the Python port publicly.

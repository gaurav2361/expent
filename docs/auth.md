# Expent Authentication & Authorization (`crates/auth`)

This document outlines the architecture, data-flow boundaries, and implementation patterns governing user authentication in the Expent ecosystem.

The system natively wraps the [Better Auth](https://github.com/better-auth/better-auth) standard within a strict Rust integration (`crates/auth`), coupling it tightly to the `axum` routing framework using the `sea_orm` database adapter.

## Architectural Overview

*   **Logic Path**: `crates/auth` -> `apps/server` -> `apps/dashboard`.
*   **Core Library**: `better_auth` (Rust Crate).
*   **Plugins Injected**: `EmailPasswordPlugin`, `SessionManagementPlugin`.
*   **Database Adapter**: A purely custom `SqliteAdapter` mapping standard `better_auth` entity traits explicitly to the `sea_orm` models (found in `crates/auth/src/adapter/`).
*   **Axum Guards**: Exposes the seamless `AuthSession` extractor trait globally for injecting securely logged-in `User` data heavily into other backend controllers.

---

## 1. Native Database Adapter (`src/adapter/`)

To achieve complete control over DB execution while staying compliant with `better_auth` specs, `crates/auth` avoids generic SQL adapters and instead implements the DB traits directly.

The `adapter/` directory enforces strict CRUD operations passing seamlessly from the BetterAuth core out to `sea_orm`:
- **`user.rs`**: Inserts and fetches against the Base `db::entities::users` table.
- **`session.rs`**: Issues timed, IP-bound active session tokens into `db::entities::sessions`.
- **`account.rs`**: Manages OAuth bindings natively into `db::entities::accounts`. 
- **`verification.rs`**: Stores short-lived OTP/verification links tightly against `db::entities::verifications`.
- **`others.rs`**: Implements required `better_auth` trait stubs for features not yet activated. All return `AuthError::NotImplemented` or empty results:
  - `OrganizationOps` / `MemberOps` / `InvitationOps` — Multi-tenant org scaffolding.
  - `TwoFactorOps` — TOTP/Backup code placeholders.
  - `ApiKeyOps` — Programmatic API key management stubs.
  - `PasskeyOps` — WebAuthn passkey storage stubs (client-side passkey *challenge* still works via the `@better-auth/passkey` JS plugin, but server-side persistence is not wired to DB yet).

---

## 2. Axum Session Extraction (`AuthSession`)

All guarded routes in `apps/server` (e.g., fetching transactions, uploading receipts) derive user context transparently by requesting `AuthSession` as an `axum` handler parameter.

### How `AuthSession` Works Internally (`src/lib.rs`)
1. **Header Interception**: Given an incoming `/api/transactions` request, `AuthSession::from_request_parts` extracts the raw HTTP headers natively.
2. **Mock Internal Request**: It compiles a pseudo HTTP `GET /get-session` request containing explicitly the user's origin headers and injects it internally into the `better_auth` state engine in-memory.
3. **Session Decoding**: The `better_auth` engine validates the JWT, checks database expiry tables, and returns a verified JSON packet resolving to the `User` struct natively.
4. **Failure Case**: Emits a `401 Unauthorized` block immediately intercepting execution before hitting the actual endpoint.

---

## 3. Configuration & Initialization

Bootstrapped within `init_auth()` mapping dynamically alongside `apps/server/src/main.rs`.

### Environment Controls
The initialization reads strict environment variables to shape security behaviors tightly without demanding re-compilation.

| Environment Variable | Impact / Default | Purpose |
|----------------------|------------------|---------|
| `BETTER_AUTH_SECRET` (or `BETTERAUTH_SECRET`) | Required | Cryptographic secret for signing/hashing JWTs. Either env var name is accepted. |
| `BETTER_AUTH_BASE_URL` (or `BASE_URL`) | Defaults `http://localhost:7878` | Essential binding URL aligning the plugin's cookie scoping logic. |
| `CORS_ORIGIN` | Appended to Trust list | Comma-separated external UI domains added to the trusted origin pool. |
| `ENABLE_SIGNUP` | Boolean (Default: `true`) | Toggles new registration logic on/off dynamically to control staging environments. |
| `REQUIRE_EMAIL_VERIFICATION`| Boolean (Default: `false`) | Toggles firm email verification handshakes. |

---

## 4. Frontend Bridging (`apps/dashboard`)

While `crates/auth` handles execution, `apps/dashboard` accesses this ecosystem using the unified TypeScript client exposed gracefully at `/api/auth`.

**Frontend Auth Client** (`src/lib/auth-client.ts`):
The dashboard creates a unified `better-auth/react` client with the following plugins:
- `passkeyClient()` — Enables WebAuthn passkey challenge flows.
- `usernameClient()` — Enables username-based lookups alongside email.

**Frontend API Endpoints Mapped Automatically**:
- `POST /api/auth/sign-in/email`
- `POST /api/auth/sign-up/email`
- `GET /api/auth/get-session`
- `POST /api/auth/sign-out`

**Rate Limit Handling**: The auth client intercepts `429` responses and logs `X-Retry-After` headers for graceful backoff.

Since both ecosystems share the strict protocol dictated by the `better_auth` specification, session headers (cookies named `better-auth.session_token` / `__Secure-better-auth.session_token`) bridge cross-origin seamlessly across environments as explicitly flagged up in the Rust compiler's trusted origin list.

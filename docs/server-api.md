# Expent Backend API Reference

This document covers the structure and available endpoints of the Expent backend server (`apps/server`). It pairs with the [Database Schema Documentation](./database_schema.md) to provide a complete overview of how data flows from the UI, through the network, to storage.

## Architectural Overview

*   **Framework**: Rust with [`axum`](https://docs.rs/axum/latest/axum/) + `tower-http` (CORS, tracing).
*   **Database ORM**: `sea_orm` (targeting SQLite).
*   **Authentication**: [`better-auth`](https://github.com/better-auth/better-auth) managed seamlessly by the router. 
*   **Base URL**: `/api` (business logic) and `/api/auth` (authentication endpoints).
*   **Default Port**: `0.0.0.0:7878` (overridable via `API_PORT` env var).
*   **Body Limit**: `10MB` globally enforced via `axum::extract::DefaultBodyLimit`.
*   **Content-Type**: Requests and Responses default to `application/json` unless otherwise specified (e.g. multipart uploads).
*   **Security Protocol**: Endpoints are strictly guarded via the `AuthSession` extractor. Requests must include authentication cookies managed by `better-auth`.

### Application State (`AppState`)
The server injects a shared `AppState` struct into all handlers:
| Field | Type | Source |
|-------|------|--------|
| `db` | `DatabaseConnection` | Connected via `DATABASE_URL` env var. |
| `auth` | `Arc<BetterAuth<SqliteAdapter>>` | Initialized via `crates/auth::init_auth()`. |
| `upload_client` | `UploadClient` | S3/R2 client from `crates/upload`. |
| `ocr_service` | `Arc<OcrService>` | Wraps the Python OCR worker URL. |

### CORS Configuration
Allowed origins: `http://localhost:3000`, `http://127.0.0.1:3000`.
Allowed headers: `Content-Type`, `Authorization`, `Accept`, `Cookie`, `x-better-auth-origin`.
Credentials: Enabled.

---

## 1. Transactions & Ledger

Endpoints managing the lifecycle of robust financial entries.

### `GET /api/transactions`
- **Purpose**: Fetch all transactions tied to the currently authenticated user.
- **Request Parameters**: None (The authenticated session implicitly defines the request boundary).
- **Response**: Array of `Transaction` objects.
- **UI Context**: Populates the "All Transactions" data table and the dashboard's "Recent Transactions" list.

### `POST /api/transactions/manual`
- **Purpose**: Log a brand new transaction manually.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `amount` | `Decimal (String/Number)` | Yes | The precise monetary value |
  | `purpose_tag` | `String` | No | Semantic category ("Groceries", "Rent") |
  | `direction` | `Enum("IN", "OUT")` | Yes | Is it an income or an expense? |
  | `date` | `DateTime (ISO 8601)` | Yes | When the transaction occurred |
- **Response**: Returns `200 OK` with the newly created `Transaction`.
- **UI Context**: Triggered from the "Add Transaction" modal form.

### `PATCH /api/transactions/:id`
- **Purpose**: Update an existing transaction's metadata dynamically. 
- **URL Params**: `:id` - UUID of the transaction.
- **Request Body** (All Optional):
  | Field | Type | Description |
  |-------|------|-------------|
  | `amount` | `Decimal` | Corrects a wrongly parsed or entered amount |
  | `date` | `DateTime` | Modifies the logging date |
  | `purpose_tag` | `String` | Updates category tags |
  | `status` | `Enum` | `PENDING`, `COMPLETED`, `CANCELLED` |
- **Response**: Returns `200 OK` with the updated `Transaction`.
- **UI Context**: Triggers from the slide-out Transaction Editor drawer form inputs.

### `DELETE /api/transactions/:id`
- **Purpose**: Remove a mislogged or duplicated transaction.
- **URL Params**: `:id` - UUID of the transaction.
- **Response**: `204 No Content`.
- **UI Context**: Triggered via a trash icon or destructive buttons in the Transaction Details view.

### `POST /api/transactions/split`
- **Purpose**: Split an otherwise isolated transaction into multiple debts.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `transaction_id` | `String` | Yes | ID of the source transaction |
  | `splits` | `Vec<SplitDetail>` | Yes | Array defining fractional allocation for target users |
- **Response**: Returns `200 OK` with an array of generated `P2PRequest` models.
- **UI Context**: Used in the "Split Transaction" widget.

---

## 2. Groups

Handles multi-user collaborative ledgers for shared events and recurring shared expenses.

### `GET /api/groups`
- **Purpose**: List all groups the active user has access to.
- **Response**: Array of `Group` models.
- **UI Context**: Populates the sidebar list of active groups and the isolated "My Groups" interface.

### `POST /api/groups/create`
- **Purpose**: Create a new shared expense tracker.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `name` | `String` | Yes | The designated title (e.g., "Trip to Bali") |
  | `description` | `String` | No | Sub-description outlining purpose |
- **Response**: Returns `200 OK` with the new `Group`.

### `POST /api/groups/invite`
- **Purpose**: Trigger an invitation to an external or internal user.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `group_id` | `String` | Yes | Target group UUID |
  | `receiver_email` | `String` | Yes | Email to route the invite out to |
- **Response**: Generates and returns `200 OK` returning a pending `P2PRequest` modeling the invite.

### `GET /api/groups/:id/transactions`
- **Purpose**: Fetch timeline expenses strictly contained within a group partition.
- **URL Params**: `:id` - UUID of the group.
- **Response**: Array of `Transaction` models filtered by `group_id`.

---

## 3. P2P & Settling

Person-to-person debt requests and tab settlements. 

### `GET /api/p2p/pending`
- **Purpose**: Retrieve pending inbound action requests (money owed, group invites) mapped to a user's identity.
- **Response**: Array of `P2PRequest` models marked as `PENDING`.
- **UI Context**: High-value route driving the UI "Notification Bell" and Dashboard Action Cards.

### `POST /api/p2p/create`
- **Purpose**: Manually formulate a direct payment link or debt assignment.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `receiver_email` | `String` | Yes | Counterparty debt holder |
  | `transaction_id` | `String` | Yes | Root transaction mapping the debt |
- **Response**: Returns `200 OK` with the finalized `P2PRequest`.

### `POST /api/p2p/accept`
- **Purpose**: Approve a P2P debt request, effectively committing to an inbound cost natively.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `request_id` | `String` | Yes | The P2P UUID to mark APPROVED/MAPPED |
- **Response**: `200 OK` outlining the updated `P2PRequest` object.

---

## 4. OCR & File Processing

Pipelines for S3/R2 direct uploads alongside Gemini inference triggers.

### `POST /api/upload/presigned`
- **Purpose**: Acquire a short-lived PUT URL from Cloudflare R2 / S3 tailored to let users upload natively rather than streaming through server compute.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `contentType` | `String` | Yes | e.g. `image/jpeg`, `application/pdf` |
  | `fileName` | `String` | Yes | `receipt_x.jpg` (Used to hash an internal storage key) |
- **Response Body**:
  ```json
  {
    "url": "https://bucket.r2.cloudflarestorage.com/signature...",
    "key": "user_id/uuid_receipt_x.jpg"
  }
  ```

### `POST /api/upload`
- **Purpose**: Alternative `multipart/form-data` upload receiver endpoint handled safely via Axum streams. 
- **Payload Constraints**: Accepts `multipart` chunk bounded to a maximum file limit (`10MB`). Key field: `file`.
- **Response**: Returns the R2 URL string and file `key`.

### `POST /api/process-image-ocr`
- **Purpose**: Commands the server to grab a target file from storage and pass it through the Google Gemini Vision model for schema-conforming parsing.
- **Security Check**: This explicitly evaluates if the payload `key` is uniformly prefixed with `<user_id>/`—if not, aborts to `403 Forbidden` to neutralize IDOR attacks.
- **Request Body**:
  | Field | Type | Required | Description |
  |-------|------|----------|-------------|
  | `key` | `String` | Yes | Remote bucket key (e.g. `usr_.../file.jpg`) |
- **Response**: Will return a parsed and populated DB `Transaction` object (Wait times factor Gemini processing). Handles `429 TOO MANY REQUESTS` dynamically if quotas trigger.

### `POST /api/process-ocr`
- **Purpose**: Consumes a structured JSON extraction packet and bridges it fully to the backing generic `Transaction` and `PurchaseItems` architecture.
- **Request Body**: Raw `db::ProcessedOcr` JSON.
- **Response**: Returns `200 OK` attached to the populated `Transaction`.

---

## 5. Subscriptions

### `GET /api/subscriptions/detect`
- **Purpose**: Perform an algorithmic pass traversing historical bank statements and transaction patterns aiming to surface recurring charges.
- **Response**: Array of candidate `Subscription` models.
- **UI Context**: Found powering the predictive Subscriptions Interface page.

---

## 6. Authentication (Better Auth)

Endpoint handling operates dynamically traversing `/api/auth/*` powered entirely by the unified Rust/TS `better-auth` integration logic.

- Handlers dynamically intercept `GET`/`POST` commands across routes like `/login`, `/register`, `/session`, and `/logout`.
- Exposes critical unified configurations mapping UI boundaries to Database `sessions`/`users`/`accounts`. 
- **Ref**: Check the [Authentication Best Practices Skill](../.agents/skills/better-auth-best-practices/SKILL.md) for full interface insights.

# Expent

Expent is an intelligent expense management platform built with Rust, TypeScript, and Python. It features OCR-based receipt ingestion, automated subscription detection, and shared ledgers for group expense tracking.

## Architecture

- **`apps/server` (Rust/Axum):** Unified backend handling authentication (Better Auth), database orchestration (SeaORM), and S3/R2 presigned URLs.
- **`apps/web` (TypeScript/TanStack):** Modern React frontend using TanStack Router, Start, and Query.
- **`apps/ocr-worker` (Python/FastAPI):** OCR processing engine using EasyOCR and pdfplumber.
- **`packages/ui`:** Shared component library built with Tailwind CSS and Shadcn.
- **`packages/types`:** Shared TypeScript types automatically generated from Rust models via `ts-rs`.

## Prerequisites

- **Node.js:** v24 or higher (pnpm recommended)
- **Rust:** Latest stable version
- **Python:** v3.13 or higher (using `uv` for dependency management)
- **Database:** SQLite (local development) or PostgreSQL (production)
- **Storage:** Cloudflare R2 or S3-compatible storage

## Getting Started

1. **Clone the repository**
2. **Install dependencies:**
   ```bash
   pnpm install
   uv sync
   ```
3. **Configure environment variables:**
   Copy `.env.example` to `.env` in the root and fill in your credentials.
   ```bash
   cp .env.example .env
   ```
   *Note: For local development with SQLite, the default `DATABASE_URL` will automatically create `expent.db` in the project root.*

4. **Initialize database and run migrations:**
   ```bash
   # From the project root
   cargo run --package migration -- up
   ```
5. **Start development server:**
   ```bash
   pnpm dev
   ```

## Key Features

- **Smart Merge:** Automatically deduplicates transactions by matching OCR results with existing bank records.
- **Itemized Splits:** Automatically parse receipt line items and split them across shared ledgers.
- **Subscription Engine:** Detects recurring payment patterns and alerts users of upcoming renewals.
- **Group Ledgers:** Collaborative spaces for tracking expenses with friends and family.

## Environment Variables

| Variable | Description |
| :--- | :--- |
| `DATABASE_URL` | PostgreSQL connection string |
| `AUTH_SECRET` | 32+ character secret for authentication |
| `S3_ENDPOINT` | S3-compatible API endpoint |
| `S3_ACCESS_KEY_ID` | Access key for storage |
| `S3_SECRET_ACCESS_KEY` | Secret key for storage |
| `S3_BUCKET_NAME` | Name of the bucket for uploads |
| `OCR_WORKER_URL` | URL where the Python worker is running |

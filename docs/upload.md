# Expent File Processor & Upload Architecture (`crates/upload`)

This document outlines the architecture, data-processing bounds, and storage integration implemented strictly within the backend's standalone `crates/upload` Rust library.

The `crates/upload` module operates as a deeply decoupled engine bridging the central Rust API Server to S3-compatible remote blob storage (e.g., AWS S3, Cloudflare R2, MinIO) while natively shielding the ecosystem from malicious file types or path traversals.

## Architectural Overview

*   **Logic Path**: `apps/server` -> `crates/upload` -> `S3 Bucket`.
*   **Core SDK**: `aws_sdk_s3` + `aws-config` (Natively async, pure Rust AWS client).
*   **Media Processing**: `image` crate (for standardizing image matrices).
*   **Security Modules**: `infer` crate (for deep byte-level MIME inspections).
*   **Serialization**: `serde` + `serde_bytes` (for efficient binary payload serialization).
*   **Error Handling**: `thiserror` (for deriving strongly-typed `UploadError` variants).

### Error Types (`UploadError`)
| Variant | Trigger |
|---------|---------|
| `ImageError` | Corrupt or unreadable image bytes passed to the `image` crate. |
| `UnknownFileType` | File categorization completely failed across all tiers. |
| `Io` | Filesystem or buffer I/O failures. |
| `S3Error` | Any AWS SDK call failure (permissions, network, bucket not found). |
| `Internal` | Catch-all for presigning config errors or body collection failures. |

---

## 1. Storage Integrations (`UploadClient`)

The core execution wrapper. Initialized once globally in `apps/server` and injected into controllers.

### `upload_direct`
- Ingests raw memory bytes (`bytes::Bytes`).
- Invokes the `UploadProcessor` natively inline to sanitize bytes before assigning a permanent unique `key`.
- Key structure: `{user_id}/{uuid}-{sanitized_filename}`.

### `get_presigned_url`
- Provides secure natively expiring URLs for uploading massive data blobs (preventing the Rust server from maxing disk limits acting as a middleman proxy).
- Mitigates path traversal securely: Drops `../` from filenames using strict Rust `Path::new().file_name()` projections.

### `get_file`
- Natively downloads raw bytes from S3. Useful for streaming securely back directly through `axum` responses matching session owners.

---

## 2. File Categorization Matrix

Before allowing *any* bytes into storage or down the OCR pipeline dynamically, the `UploadProcessor::process` logic executes a strictly ordered tiered inspection to guarantee categorization.

### The Categorization Flow (`determine_category`)
1. **Content-Type Claim**: Evaluates HTTP Headers (Easily spoofed).
2. **File Extension Regex**: Checks the URL/FileName extension natively.
3. **Magic Byte Analysis (The Truth Source)**: Utilizes the `infer` crate to read the first literal byte matrices of the raw payload to confirm the true file structure, bypassing all spoof attempts.

### Available Mappings (`FileCategory`)
| Enum State | Handled Types | Post-Processing |
|------------|--------------|-----------------|
| `Image` | `.png`, `.jpeg`, `.webp`, `.heic` | Will convert *all* formats down reliably to lossless `PNG` if image normalization is enabled (e.g., via a configuration flag passed to the `UploadProcessor` or an environment variable). This ensures a consistent input format for downstream services like the OCR pipeline. |
| `Pdf` | `application/pdf`, `%PDF` magic string | Ingested purely. Mapped towards PDF scraping flows. |
| `Csv` | Text, `application/csv` | Ingested natively. Validates UTF-8 boundaries implicitly. |
| `Unknown` | All other structures. | Explicitly flagged `Unknown` mitigating code injections. |

---

## 3. Image Normalization

A unique strict utility embedded in `crates/upload`.
When standardizing data for the Python `/ocr` pipeline internally:
- `UploadProcessor::convert_to_png(data)` forces complex/unsupported byte formats like `.heic` (from iPhones) or corrupted `.jpeg` matrices into standardized contiguous `.png` bytes stored in memory before ingestion.
- This prevents the external FastAPI (Python) worker from encountering unparsable payloads and throwing unhandled `500` faults.

**Internal Processing Flow**: Raw bytes enter as `Bytes` → `UploadProcessor::process()` produces a `RawProcessedFile` (intermediate, in-memory only) → `UploadClient::upload_direct()` pushes to S3 and wraps it as the final `ProcessedFile` (which includes the assigned `key`).

---

## 4. Built-in Test Suite

The crate ships with unit tests (`#[cfg(test)]`) covering critical security and categorization logic:

| Test | Validates |
|------|-----------|
| `test_determine_category_pdf` | Detects `%PDF` magic bytes correctly. |
| `test_determine_category_image` | Detects PNG magic bytes (`\x89PNG`). |
| `test_determine_category_csv` | Validates both extension-based and content-type-based CSV detection. |
| `test_determine_category_unknown` | Confirms unknown payloads are safely flagged as `Unknown`. |
| `test_s3_key_path_traversal_mitigated` | Verifies that `../` directory traversal in filenames is stripped by `Path::new().file_name()`, preventing S3 key manipulation attacks. |

# OCR Crate Architecture Documentation

This document provides a detailed breakdown of the `ocr` library crate located at `crates/ocr`. It outlines the crate's purpose and its interaction with the external Python data extraction worker.

---

## 1. Overview & Purpose

The `ocr` crate acts as a lightweight, non-blocking HTTP proxy client. Instead of bundling complex Python, PyTorch, and AI dependencies into the core Rust server container, the actual OCR capabilities are hosted externally in `apps/ocr`. This crate securely bridges the gap.

---

## 2. Core Mechanisms

### `OcrService`
A struct harboring an optimized `reqwest::Client`.
- It manages connection pooling and resolves the appropriate internal network URL for the OCR worker (e.g., `http://localhost:8090/extract` fallback).
- It is initialized once during server startup and stored inside Axum's Application State.

### `process_file`
The primary method exposed to the server.
- It accepts the raw binary data (`file_bytes`), the `filename`, and the expected `mime_type` natively from an HTTP multipart upload.
- It instantly converts these primitives into a `multipart::Form` compatible with Python FastAPI standards.
- It shoots the request across the network, awaits the JSON schema from the Python worker, and yields a generic `serde_json::Value` payload back to the Rust server.

---

## 3. Integration Flow

1. A user uploads a receipt image via the web client.
2. The `apps/server` upload handler receives the file bytes.
3. The server invokes `state.ocr_service.process_file(...)`.
4. This crate POSTs the multipart data to the `apps/ocr` container.
5. The `apps/ocr` worker processes the receipt (using EasyOCR + Gemini).
6. The JSON structure is returned to the crate.
7. The crate bubbles it back to the server, where it is mapped into the `SmartMerge` database logic.

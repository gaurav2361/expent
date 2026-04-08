# Expent OCR & Extraction API Reference

This document covers the structure and available endpoints of the internal OCR microservice (`apps/ocr`). It operates strictly as an internal data-processing pipeline consumed by the **`expent_core`** hub via the **`apps/api`** gateway.

## Architectural Overview

- **Framework**: Python with [`FastAPI`](https://fastapi.tiangolo.com/)
- **Worker Engine**: `core.py` powering [Google Gemini 2.5 Flash GenAI](https://ai.google.dev/) + [EasyOCR](https://github.com/JaidedAI/EasyOCR).
- **Rust-Side Orchestration**: Managed by **`expent_core::services::ocr`**, which handles background job status, data mapping, and database persistence.
- **Base URL**: `http://localhost:8090` (internal).
- **Security Protocol**: Private microservice. The **`apps/api`** acts as an authenticating gateway, while **`expent_core`** ensures requests have passed authorization boundaries.

---

## 1. Extraction Pipeline

The primary endpoint triggering document classification and structured JSON transformation.

### `POST /extract`

- **Purpose**: Intake raw bytes of documents and transform them into precise JSON fields.
- **Request Format**: `multipart/form-data`
- **Request Logic Bounds**:
  | Field | Type | Constraint | Description |
  |-------|------|----------|-------------|
  | `file` | `UploadFile` (bytes) | strictly `< 20MB` | Accepts Images (`.png`, `.jpg`), `.pdf`, and `.csv`. |

#### Processing Pipeline Architecture:

1. **Validation**: Size checks and DoS protection.
2. **Text Pre-Pass (EasyOCR)**: Provides Gemini with computer vision context.
3. **Classification**: Document identified as `GPAY` or `GENERIC`.
4. **Structured Extraction**: Gemini Schema conforming ensures deterministic JSON output.

---

## 2. Response Formats

Responses follow strict schemas parsed into `expent_core::ProcessedOcr`.

### Response `200 OK` (Format Structure)

```json
{
  "doc_type": "GPAY" | "GENERIC",
  "data": { ... }
}
```

#### Scenario A: `doc_type: "GPAY"` (Google Pay / UPI)

Maps to `expent_core::GPayExtraction`. Includes amount, direction, counterparty details, and transaction IDs.

#### Scenario B: `doc_type: "GENERIC"` (Invoices / Receipts)

Maps to `expent_core::OcrResult`. Used to populate `purchases` and `purchase_items` tables via the core orchestration layer.

---

## 3. Worker Diagnostics & Health

### `GET /`

- **Response Format**: `200 OK`
  ```json
  {
    "status": "healthy",
    "service": "ocr"
  }
  ```

---

## HTTP Failures & Handling

| Code  | Trigger                    | Action Logic                                            |
| ----- | -------------------------- | ------------------------------------------------------- |
| `413` | Payload exceeded `20MB`.   | Reject upload flow.                                     |
| `429` | Gemini Quota Exceeded.     | **`expent_core`** surfaces "Quota Exceeded" gracefully. |
| `500` | Internal processing error. | Abort with generic backend fault flags.                 |

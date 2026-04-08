# Expent OCR & Extraction API Reference

This document covers the structure and available endpoints of the internal OCR microservice (`apps/ocr`). It operates strictly as an internal data-processing pipeline consumed by the robust Rust Backend Server (`apps/server`) and never interacts natively with the UI or End Users.

## Architectural Overview

*   **Framework**: Python with [`FastAPI`](https://fastapi.tiangolo.com/)
*   **Worker Engine**: `core.py` powering [Google Gemini 2.5 Flash GenAI](https://ai.google.dev/) + [EasyOCR](https://github.com/JaidedAI/EasyOCR).
*   **Base URL**: `http://localhost:8090` (defaults to private networks or Docker routing clusters).
*   **Content-Type**: Receives `multipart/form-data` uploads. Returns structured `application/json`.
*   **Security Protocol**: Private microservice. The Rust server acts as an authenticating gateway and assumes the requests reaching the OCR worker have passed IDOR, Quota checks, and authorization boundaries.

---

## 1. Extraction Pipeline

The primary endpoint triggering hybrid Image/PDF ingestion, document classification, text scraping (via EasyOCR pipeline backups), and structured JSON transformation (via Gemini Schema conforming).

### `POST /extract`
- **Purpose**: Intake raw bytes of documents/imagery and transform them into precise JSON fields tailored to the exact document type.
- **Request Format**: `multipart/form-data`
- **Request Logic Bounds**:
  | Field | Type | Constraint | Description |
  |-------|------|----------|-------------|
  | `file` | `UploadFile` (bytes) | strictly `< 20MB` | The actual file object. Accepts Images (`.png`, `.jpg`), `.pdf` files, and raw `.csv` bank dumps. |

*(Note: The legacy endpoint `POST /ocr` points explicitly to `/extract` for backward compatibility).*

#### Processing Pipeline Architecture:
1. **Validation & DoS Protection**: Hardcoded checks abort workloads `>20MB` natively raising a `413 Payload Too Large`. A secondary `len(data)` check runs post-read to catch spoofed `file.size` headers.
2. **Text Pre-Pass (EasyOCR)**: Triggers an initial computer vision pass over imagery to supply Gemini with explicit context. The EasyOCR `Reader` is **lazy-loaded** on first use to avoid startup latency when the worker boots.
3. **Classification**: Gemini natively classifies the document into explicitly tagged branches:
   - `GPAY`: Triggers `GPAY_SYSTEM_PROMPT` specifically tuned to identify UPI targets.
   - `GENERIC`: Triggers a fallback `GENERIC_SYSTEM_PROMPT` designed for generic retail receipts.
4. **Structured Extraction**: Gemini is called with `temperature=0.0` and `response_mime_type="application/json"` using the Pydantic schema as `response_schema`, guaranteeing deterministic, parseable JSON output.

---

## 1.1 Utility Functions (`utils.py`)

Helper functions used throughout the extraction pipeline:

| Function | Purpose |
|----------|---------|
| `get_media_type(filename)` | Maps file extensions to MIME types (`.png` → `image/png`, `.pdf` → `application/pdf`, `.csv` → `text/csv`). Defaults to `image/png`. |
| `rasterize_pdf_page(pdf_bytes, page_num, dpi)` | Converts a single PDF page into PNG bytes using **PyMuPDF** (`fitz`) for downstream vision processing. |
| `extract_pdf_text(pdf_bytes)` | Extracts raw text from all PDF pages using **pdfplumber** as a text-layer fallback. |
| `parse_csv(csv_bytes)` | Decodes CSV bytes to UTF-8 and returns a `list[dict]` via Python's `csv.DictReader`. |

---

## 2. Response Formats

The JSON responses natively resolve into two different strict schemas heavily dependent on classification branching. Both schemas are parsed gracefully into `db::ProcessedOcr` on the Rust server.

### Response `200 OK` (Format Structure)

The outer layer always follows:
```json
{
  "doc_type": "GPAY" | "GENERIC",
  "data": { ... }
}
```

#### Scenario A: `doc_type: "GPAY"` (Google Pay / UPI Processing)
Maps to the `GPayExtraction` Pydantic class.

| Extracted Field | Output Type | Description |
|-------|------|-------------|
| `amount` | `Float` | Conclusively parsed financial transfer amount |
| `direction` | `Enum("IN", "OUT")` | Derives whether money left ("To") or arrived ("From") |
| `datetime_str` | `String (Optional)` | The raw datestring block |
| `status` | `Enum` | `COMPLETED`, `PENDING`, string matches |
| `counterparty_name` | `String` | Full name or merchant title of interacting party |
| `counterparty_phone` | `String (Optional)` | Pulled dynamically from screens |
| `counterparty_upi_id` | `String (Optional)` | UPI strings containing `@` |
| `is_merchant` | `Boolean` | True if keyword scanning denotes a retail target / business terminal |
| `source_bank_account` | `String (Optional)` | Last 4 digits + Bank String (e.g. "HDFC 1234") |
| `upi_transaction_id` | `String (Optional)` | E.g. "T230..." |
| `google_transaction_id` | `String (Optional)` | GPay specific strings |

#### Scenario B: `doc_type: "GENERIC"` (Standard Invoices / Receipts)
Maps to the `OCRResponse` Pydantic class. This response is often used to populate the `purchase_imports` and subsequently `purchases` and `purchase_items` tables (see `docs/database_schema.md`).

| Extracted Field | Output Type | Description |
|-------|------|-------------|
| `raw_text` | `String` | Fallback aggregate blob from EasyOCR + Gemini |
| `vendor` | `String (Optional)` | Detected or user-specified vendor, often inferred from receipt headers/footers or explicit document classification. Used for `purchase_imports` (see `docs/database_schema.md`). |
| `amount` | `Float (Optional)` | Total transaction mass/sum matched over the receipt lines |
| `date` | `String (Optional)` | Receipt header dates |
| `upi_id` | `String (Optional)` | Extracted if heavily noted in footers |
| `document_type` | `Enum` | Determines: `payment_receipt` / `invoice` / `bank_statement` / `other` |
| `confidence` | `Enum` | Determines inference trust: `high` / `medium` / `low` |
| `items` | `Array<LineItem>` | A mapped array of nested LineItem structured rows |

**`LineItem` Inner Object Schema:**
| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `name` | `String` | Recognizable item title | `required` |
| `quantity` | `Integer` | Fallback to `1` natively if unset | `1` |
| `sku` | `String (Optional)` | Stock Keeping Unit / barcode | `optional` |

---

## 3. Worker Diagnostics & Health

### `GET /`
- **Purpose**: Orchestrators (Docker, Kubernetes, or the Rust Server) poll this route to assert readiness limits before queuing extractions.
- **Response Format**: `200 OK`
  ```json
  {
    "status": "healthy", 
    "service": "ocr"
  }
  ```

---

## Common HTTP Failures 

Expected handling rules across the Rust server mapping boundaries.

| Code | Trigger | Action Logic |
|-------|------|----------|
| `413` | Upload payload exceeded `20MB` limit natively | Reject upload flow immediately |
| `429` | Gemini Tier limits / Rate limits encountered on worker (`genai.Client`) | Retrigger delay or surface "Quota Exceeded" to users directly gracefully |
| `500` | Fallback for arbitrary JSON unparsing or corrupted binary handling | Abort processing gracefully with generic backend fault flags |

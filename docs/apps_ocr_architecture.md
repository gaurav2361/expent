# OCR App Architecture Documentation

This document examines the dedicated Python extraction microservice located at `apps/ocr`. Written in FastAPI, this application powers document intelligence.

---

## 1. Overview & Purpose

Processing matrices, parsing PDFs, or interacting with large language models require specific dependencies (like PyTorch or Google Generative AI SDKs). By isolating these mechanics outside of the `apps/server` Rust process, the architecture achieves a smaller footprint per container, prevents catastrophic C++ binding crashes in the core router, and allows the OCR compute cluster to horizontally autoscale independent of the web traffic.

---

## 2. Core Extractor Logic (`OCREngine`)

At the heart of the service resides the `OCREngine` which uses a hybrid cascade strategy.

### Tier 1: Context Harvesting
Traditional heuristic parsers remain highly effective.
- **PDFs**: If a `application/pdf` is sent, `OCREngine` uses `pdfplumber`/`PyPDF2` algorithms natively.
- **Images**: If it's a graphical format, it spins up `easyocr` (ideally GPU constrained) to detect all raw text bounding boxes across the X / Y axis.
- **CSVs**: Sniffed completely locally using pandas.

### Tier 2: Vision Model Re-Structure
OCR text dumps are inherently unstructured and hard to query.
- The `google.generativeai` package bridges this.
- Providing Gemini (`gemini-2.5-flash` typically) the original image/PDF binary alongside the **Harvested Context** generated from Tier 1 heavily reduces hallucination rates. 
- Using predefined prompt instructions (`SYSTEM_PROMPT`), the Python runtime demands a strict serialization conforming to Expent's expected format (detecting `grand_total`, `date`, `vendor`, and `amount`).

---

## 3. Communication Boundary

The app opens a single explicit `/extract` routing endpoint on `port 8090` exposed to the internal network.
The Rust backend posts unparsed image bytes here; the worker returns a predictable, mapped JSON string representing the financial transaction context, completely abstracting the complex AI parsing away from `apps/server`.

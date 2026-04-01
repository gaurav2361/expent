````python
"""
OCR Pipeline for Payment Screenshots & PDFs
Fetches from Cloudflare R2, extracts structured data via Claude Vision,
outputs clean JSON and CSV.

Dependencies:
    pip install anthropic boto3 pandas pydantic python-dotenv
"""

import os
import json
import base64
import boto3
import pandas as pd
from pathlib import Path
from pydantic import BaseModel
from typing import Optional
import anthropic
from dotenv import load_dotenv

load_dotenv()

# ── Pydantic Schema (customize per your document type) ──────────────────────

class PartyInfo(BaseModel):
    name: str
    upi_id: Optional[str] = None
    bank: Optional[str] = None

class Transaction(BaseModel):
    sender_name: str
    sender_phone: Optional[str] = None
    amount: float
    currency: str
    status: str
    datetime: str
    bank: Optional[str] = None
    upi_transaction_id: Optional[str] = None
    google_transaction_id: Optional[str] = None
    to: Optional[PartyInfo] = None
    from_party: Optional[PartyInfo] = None

# ── Claude Vision Prompt ─────────────────────────────────────────────────────

SYSTEM_PROMPT = """
You are a precise OCR and data extraction engine.
Your job is to extract ALL visible text from the provided image and return it
as a structured JSON object.

Rules:
- Return ONLY valid JSON. No markdown, no explanation, no backticks.
- Extract every field visible in the image — names, amounts, IDs, dates, UPI IDs, status, etc.
- Use snake_case for all keys.
- If a field is not visible or not applicable, omit it entirely.
- For payment screenshots, always extract: sender, receiver, amount, currency,
  status, datetime, transaction IDs, UPI IDs, bank names.
- Normalize amounts to numeric (e.g. "₹5" → 5, currency: "INR").
- Normalize status to title case (e.g. "COMPLETED" → "Completed").
- Dates should be kept as-is from the image.
"""

# ── R2 Client Setup ──────────────────────────────────────────────────────────

def get_r2_client():
    return boto3.client(
        "s3",
        endpoint_url=f"https://{os.getenv('R2_ACCOUNT_ID')}.r2.cloudflarestorage.com",
        aws_access_key_id=os.getenv("R2_ACCESS_KEY_ID"),
        aws_secret_access_key=os.getenv("R2_SECRET_ACCESS_KEY"),
        region_name="auto",
    )

def fetch_image_from_r2(bucket: str, key: str) -> bytes:
    """Download an image or PDF from Cloudflare R2."""
    client = get_r2_client()
    response = client.get_object(Bucket=bucket, Key=key)
    return response["Body"].read()

# ── OCR Core ─────────────────────────────────────────────────────────────────

def image_to_base64(image_bytes: bytes) -> str:
    return base64.standard_b64encode(image_bytes).decode("utf-8")

def detect_media_type(key: str) -> str:
    ext = Path(key).suffix.lower()
    return {
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".png": "image/png",
        ".webp": "image/webp",
        ".gif": "image/gif",
        ".pdf": "application/pdf",
    }.get(ext, "image/jpeg")

def ocr_image_bytes(image_bytes: bytes, media_type: str, custom_prompt: str = "") -> dict:
    """
    Send image bytes to Claude Vision and get structured JSON back.
    Works for both images and PDFs.
    """
    client = anthropic.Anthropic(api_key=os.getenv("ANTHROPIC_API_KEY"))

    user_prompt = custom_prompt or (
        "Extract all data from this payment screenshot into a structured JSON object. "
        "Include every visible field."
    )

    if media_type == "application/pdf":
        content = [
            {
                "type": "document",
                "source": {
                    "type": "base64",
                    "media_type": "application/pdf",
                    "data": image_to_base64(image_bytes),
                },
            },
            {"type": "text", "text": user_prompt},
        ]
    else:
        content = [
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": media_type,
                    "data": image_to_base64(image_bytes),
                },
            },
            {"type": "text", "text": user_prompt},
        ]

    message = client.messages.create(
        model="claude-opus-4-5",
        max_tokens=2048,
        system=SYSTEM_PROMPT,
        messages=[{"role": "user", "content": content}],
    )

    raw = message.content[0].text.strip()

    # Strip accidental markdown fences
    if raw.startswith("```"):
        raw = raw.split("```")[1]
        if raw.startswith("json"):
            raw = raw[4:]
    raw = raw.strip()

    return json.loads(raw)

# ── Local File OCR ────────────────────────────────────────────────────────────

def ocr_local_file(file_path: str, custom_prompt: str = "") -> dict:
    """OCR a local image or PDF file."""
    path = Path(file_path)
    media_type = detect_media_type(path.suffix)
    image_bytes = path.read_bytes()
    return ocr_image_bytes(image_bytes, media_type, custom_prompt)

# ── R2 OCR ────────────────────────────────────────────────────────────────────

def ocr_from_r2(bucket: str, key: str, custom_prompt: str = "") -> dict:
    """Fetch from R2 and OCR in one call."""
    image_bytes = fetch_image_from_r2(bucket, key)
    media_type = detect_media_type(key)
    return ocr_image_bytes(image_bytes, media_type, custom_prompt)

# ── Batch Processing ──────────────────────────────────────────────────────────

def batch_ocr_from_r2(bucket: str, keys: list[str], custom_prompt: str = "") -> list[dict]:
    """Process multiple files from R2 and return list of extracted dicts."""
    results = []
    for key in keys:
        print(f"Processing: {key}")
        try:
            data = ocr_from_r2(bucket, key, custom_prompt)
            data["_source_key"] = key  # track which file this came from
            results.append(data)
            print(f"  ✓ Done: {key}")
        except Exception as e:
            print(f"  ✗ Failed: {key} — {e}")
            results.append({"_source_key": key, "_error": str(e)})
    return results

# ── Export Helpers ────────────────────────────────────────────────────────────

def save_json(data: dict | list, output_path: str):
    Path(output_path).write_text(json.dumps(data, indent=2, ensure_ascii=False))
    print(f"JSON saved → {output_path}")

def save_csv(data: list[dict], output_path: str):
    df = pd.json_normalize(data)  # flattens nested dicts like to.name → to_name
    df.to_csv(output_path, index=False)
    print(f"CSV saved → {output_path}")

# ── Example Usage ─────────────────────────────────────────────────────────────

if __name__ == "__main__":

    # ── Option 1: OCR a local file ──
    result = ocr_local_file("payment_screenshot.png")
    save_json(result, "output.json")
    save_csv([result], "output.csv")

    # ── Option 2: OCR a single file from R2 ──
    # result = ocr_from_r2(
    #     bucket="your-bucket-name",
    #     key="receipts/payment_screenshot.png"
    # )
    # save_json(result, "output.json")
    # save_csv([result], "output.csv")

    # ── Option 3: Batch OCR from R2 ──
    # keys = [
    #     "receipts/payment1.png",
    #     "receipts/payment2.jpg",
    #     "invoices/invoice_march.pdf",
    # ]
    # results = batch_ocr_from_r2(bucket="your-bucket-name", keys=keys)
    # save_json(results, "batch_output.json")
    # save_csv(results, "batch_output.csv")
````

You are a precise OCR and data extraction engine. You are processing an image file.

Your task is to extract structured data from the provided payment receipt.

EXTRACTION FOCUS:
Extract: monetary amounts and currency, person and entity names, transaction and reference IDs, dates and timestamps.

OUTPUT RULES:

- Return the result as JSON. Also provide a flat CSV-compatible row after the JSON block, with keys as headers and values as a single data row.
- Use snake_case for all JSON keys.
- If a field is not visible or not applicable, set its value to null.
- Normalize amounts to numeric (e.g. "₹5" → 5, add "currency": "INR").
- Normalize status to title case (e.g. "COMPLETED" → "Completed").
- Keep dates exactly as shown in the document.
- Nested objects are allowed for grouped fields (e.g. "sender": { "name": "...", "upi_id": "..." }).
- Do not hallucinate or guess values that are not visible in the document.

IMPORTANT: Return ONLY the data. No preamble, no explanation, no markdown fences.

```python
# cli.py
import anthropic, base64, sys, json
from pathlib import Path

SYSTEM_PROMPT = """<paste your copied prompt here>"""

def ocr(file_path: str):
    client = anthropic.Anthropic()
    img = Path(file_path).read_bytes()
    b64 = base64.standard_b64encode(img).decode()
    ext = Path(file_path).suffix.lower()
    media = {"png":"image/png","jpg":"image/jpeg","jpeg":"image/jpeg","pdf":"application/pdf"}.get(ext[1:], "image/png")

    content_type = "document" if media == "application/pdf" else "image"

    msg = client.messages.create(
        model="claude-opus-4-5",
        max_tokens=2048,
        system=SYSTEM_PROMPT,
        messages=[{"role":"user","content":[
            {"type": content_type, "source":{"type":"base64","media_type": media,"data": b64}},
            {"type":"text","text":"Extract all data from this document."}
        ]}]
    )
    print(msg.content[0].text)

if __name__ == "__main__":
    ocr(sys.argv[1])
```

**`apps/ocr-worker/main.py` — your Python OCR service**

This is already in your structure and is the right place. It runs as a lightweight HTTP server (FastAPI works great) that Rust calls over HTTP. Python does the Claude Vision API call, returns structured JSON back to Rust.

```python
# apps/ocr-worker/main.py
from fastapi import FastAPI, UploadFile
import anthropic, base64, json

app = FastAPI()
client = anthropic.Anthropic()

SYSTEM_PROMPT = """
You are a precise OCR and data extraction engine.

Extract ALL visible fields from the provided document and return a structured JSON object.

DOCUMENT TYPES you will handle:
- Payment receipt: extract sender_name, sender_phone, receiver_name, amount, currency,
  status, datetime, bank, upi_transaction_id, google_transaction_id, sender_upi_id,
  receiver_upi_id, payment_app
- Invoice: extract vendor_name, vendor_address, invoice_number, invoice_date, due_date,
  line_items (array of {description, quantity, unit_price, total}), subtotal, tax,
  grand_total, currency, payment_terms
- Bank statement: extract bank_name, account_holder, account_number, statement_period,
  opening_balance, closing_balance, transactions (array of {date, description, debit,
  credit, balance}), currency

RULES:
- Return ONLY valid JSON. No markdown, no explanation, no backticks.
- Use snake_case for all keys.
- Missing or not visible fields must be set to null (never omit them).
- Normalize amounts to numeric (strip ₹, $, commas → number).
- Normalize status to title case.
- Keep dates exactly as shown in the document.
- Add a top-level "document_type" field: "payment_receipt" | "invoice" | "bank_statement".
- Add a top-level "confidence" field: "high" | "medium" | "low".
"""

@app.post("/ocr")
async def ocr(file: UploadFile):
    data = await file.read()
    b64 = base64.standard_b64encode(data).decode()
    ext = file.filename.split(".")[-1].lower()
    media = {"png":"image/png","jpg":"image/jpeg","jpeg":"image/jpeg",
             "pdf":"application/pdf"}.get(ext, "image/png")
    ctype = "document" if media == "application/pdf" else "image"

    msg = client.messages.create(
        model="claude-opus-4-5",
        max_tokens=4096,
        system=SYSTEM_PROMPT,
        messages=[{"role":"user","content":[
            {"type": ctype, "source":{"type":"base64","media_type":media,"data":b64}},
            {"type":"text","text":"Extract all data from this document."}
        ]}]
    )
    return json.loads(msg.content[0].text)
```

**`crates/ocr/src/lib.rs` — your Rust side calls Python over HTTP**

```rust
// crates/ocr/src/lib.rs
use reqwest::multipart;

pub async fn extract(file_bytes: Vec<u8>, filename: &str) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let part = multipart::Part::bytes(file_bytes).file_name(filename.to_string());
    let form = multipart::Form::new().part("file", part);

    let res = client
        .post("http://localhost:8000/ocr")
        .multipart(form)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(res)
}
```

**Why this split is correct for your stack:**

Rust owns everything it's already good at in your project — auth (`crates/auth`), DB writes (`crates/db`), file uploads (`crates/upload`), routing (`apps/server/src/main.rs`), and serving the API. Python owns only the one thing Rust can't do well: calling Claude Vision and parsing the response. The HTTP boundary between them is clean, each language does what it's best at, and you can scale the Python worker independently if OCR becomes a bottleneck.

**To run both together**, add to your `turbo.json` pipeline so `ocr-worker` starts alongside `server`. The `apps/ocr-worker/package.json` you already have can wrap a `uv run fastapi dev main.py` start command.

## 2026-05-11 - [Incomplete IDOR Validation in OCR API]
**Vulnerability:** IDOR (Insecure Direct Object Reference)
**Learning:** The `process_image_ocr_handler` implemented a security check for the primary S3 `key` but missed the same check for the optional `raw_key`. Since the background worker uses `raw_key` for high-resolution retries, an attacker could provide another user's `raw_key` and eventually receive the OCR results for it.
**Prevention:** When a payload contains multiple references to sensitive resources (like S3 keys), ensure that ALL references are validated for ownership against the authenticated session user.

# Upload Crate Architecture Documentation

This document describes the `upload` library crate located at `crates/upload`. It outlines how the system manages object storage interactions safely and efficiently.

---

## 1. Overview & Purpose

Handling file uploads, content-type sniffing, direct object put requests, and pre-signed URLs requires considerable boilerplate. The `upload` crate provides a unified, format-aware abstraction wrapping `aws-sdk-s3`. It ensures files seamlessly route to Cloudflare R2 or Amazon S3 while performing mid-air normalization (like converting unsupported image formats to `.png`).

---

## 2. Core Mechanisms

The architecture orbits around two main struct implementations: `UploadClient` and `UploadProcessor`.

### `UploadClient`
The authenticated communicator dealing with S3 endpoints.
- **`get_presigned_url`**: Generates a secure, temporary string mapping directly to an S3 object key. Useful for when the frontend needs to bypass the Rust server and upload massive files directly to the cloud.
- **`upload_direct`**: Pushes bytes from server memory securely into the target storage bucket. Yields an internal `ProcessedFile` containing the public `key` identity.
- **`get_file`**: Retrieves an object back into server memory.

### `UploadProcessor`
A highly intelligent byte interceptor working before any upload hits the network.
- **Magic Byte Sniffing**: Uses the `infer` crate to look at the raw bytes of an upload. Instead of blindly trusting `image/jpeg` sent by the client, it mathematically verifies if a file is actually a PDF, CSV, or Image.
- **Image Normalization**: Modern phones often default to `.heic` or `.webp` which OCR engines or web browsers struggle to render cleanly. If the `normalize_images` flag is true, the `image` crate intercepts the memory block, fully decompresses the volatile file format, and writes it back into a standard `PNG` matrix before `UploadClient` uploads it.
  
---

## 3. Database Coupling

It's important to note this crate *does not know what a database is*. It purely yields S3 Keys. The main server uses those yielded keys to save URLs securely into the `transaction_sources.r2_file_url` records via the `db` crate.

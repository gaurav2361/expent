import base64
import fitz  # PyMuPDF

import base64
import csv
import io
import fitz  # PyMuPDF
import pdfplumber


def get_media_type(filename: str) -> str:
    ext = filename.split(".")[-1].lower() if filename else "png"
    return {
        "png": "image/png",
        "jpg": "image/jpeg",
        "jpeg": "image/jpeg",
        "pdf": "application/pdf",
        "csv": "text/csv",
    }.get(ext, "image/png")


def to_base64(data: bytes) -> str:
    return base64.standard_b64encode(data).decode()


def rasterize_pdf_page(pdf_bytes: bytes, page_num: int = 0, dpi: int = 150) -> bytes:
    """Convert a single PDF page into PNG bytes for vision processing."""
    doc = fitz.open(stream=pdf_bytes, filetype="pdf")
    page = doc.load_page(page_num)
    pix = page.get_pixmap(dpi=dpi)
    return pix.tobytes("png")


def extract_pdf_text(pdf_bytes: bytes) -> str:
    """Extract text from PDF using pdfplumber."""
    text = ""
    with pdfplumber.open(io.BytesIO(pdf_bytes)) as pdf:
        for page in pdf.pages:
            page_text = page.extract_text()
            if page_text:
                text += page_text + "\n"
    return text.strip()


def parse_csv(csv_bytes: bytes) -> list[dict]:
    """Parse CSV bytes into a list of dictionaries."""
    content = csv_bytes.decode("utf-8")
    reader = csv.DictReader(io.StringIO(content))
    return list(reader)

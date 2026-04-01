import base64
import fitz  # PyMuPDF


def get_media_type(filename: str) -> str:
    ext = filename.split(".")[-1].lower() if filename else "png"
    return {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg", "pdf": "application/pdf"}.get(
        ext, "image/png"
    )


def to_base64(data: bytes) -> str:
    return base64.standard_b64encode(data).decode()


def rasterize_pdf_page(pdf_bytes: bytes, page_num: int = 0, dpi: int = 150) -> bytes:
    """Convert a single PDF page into PNG bytes for vision processing."""
    doc = fitz.open(stream=pdf_bytes, filetype="pdf")
    page = doc.load_page(page_num)
    pix = page.get_pixmap(dpi=dpi)
    return pix.tobytes("png")

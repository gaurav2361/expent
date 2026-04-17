import pytest
import sys
import os
from unittest.mock import MagicMock, patch

# Mock dependencies that might be missing in the test environment
# This allows tests to run even if PyMuPDF or pdfplumber are not installed
sys.modules["fitz"] = MagicMock()
sys.modules["pdfplumber"] = MagicMock()

# Add parent directory to path to allow absolute imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))

from apps.ocr.utils import (
    get_media_type,
    to_base64,
    parse_csv,
    extract_pdf_text,
    rasterize_pdf_page,
)


def test_get_media_type():
    # Test valid known extensions
    assert get_media_type("image.png") == "image/png"
    assert get_media_type("image.jpg") == "image/jpeg"
    assert get_media_type("image.jpeg") == "image/jpeg"
    assert get_media_type("document.pdf") == "application/pdf"
    assert get_media_type("data.csv") == "text/csv"

    # Test case insensitivity
    assert get_media_type("IMAGE.PNG") == "image/png"
    assert get_media_type("Image.Jpg") == "image/jpeg"
    assert get_media_type("DOCUMENT.PDF") == "application/pdf"

    # Test fallback for unknown extensions
    assert get_media_type("document.txt") == "image/png"
    assert get_media_type("file.unknown") == "image/png"

    # Test fallback for no extension
    assert get_media_type("image") == "image/png"
    assert get_media_type("archive.") == "image/png"

    # Test edge cases
    assert get_media_type("") == "image/png"
    assert get_media_type(None) == "image/png"
    assert get_media_type(".hidden") == "image/png"
    assert get_media_type("multiple.dots.image.jpg") == "image/jpeg"


def test_to_base64():
    data = b"hello world"
    # base64 of "hello world" is "aGVsbG8gd29ybGQ="
    assert to_base64(data) == "aGVsbG8gd29ybGQ="
    assert to_base64(b"") == ""


def test_parse_csv():
    csv_content = b"name,age\nAlice,30\nBob,25"
    result = parse_csv(csv_content)
    assert len(result) == 2
    assert result[0] == {"name": "Alice", "age": "30"}
    assert result[1] == {"name": "Bob", "age": "25"}


def test_extract_pdf_text():
    mock_pdf = MagicMock()
    mock_page1 = MagicMock()
    mock_page1.extract_text.return_value = "Page 1 text"
    mock_page2 = MagicMock()
    mock_page2.extract_text.return_value = "Page 2 text"
    mock_pdf.pages = [mock_page1, mock_page2]

    with patch("pdfplumber.open") as mock_open:
        mock_open.return_value.__enter__.return_value = mock_pdf

        result = extract_pdf_text(b"fake pdf content")

        assert "Page 1 text" in result
        assert "Page 2 text" in result
        mock_open.assert_called_once()


def test_rasterize_pdf_page():
    mock_doc = MagicMock()
    mock_page = MagicMock()
    mock_pix = MagicMock()
    mock_pix.tobytes.return_value = b"fake png bytes"

    mock_doc.load_page.return_value = mock_page
    mock_page.get_pixmap.return_value = mock_pix

    with patch("fitz.open") as mock_open:
        mock_open.return_value = mock_doc

        result = rasterize_pdf_page(b"fake pdf content", page_num=0)

        assert result == b"fake png bytes"
        mock_open.assert_called_once()
        mock_doc.load_page.assert_called_with(0)
        mock_page.get_pixmap.assert_called()

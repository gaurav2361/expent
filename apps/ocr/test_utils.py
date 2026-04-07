import pytest
import sys
import os

# Add parent directory to path to allow absolute imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../..')))

from apps.ocr.utils import get_media_type

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

    # Test missing extensions (should fallback to png)
    assert get_media_type("image") == "image/png"

    # Test unknown extensions (should fallback to png)
    assert get_media_type("document.txt") == "image/png"
    assert get_media_type("archive.zip") == "image/png"

    # Test edge cases
    assert get_media_type("") == "image/png"
    assert get_media_type(None) == "image/png"
    assert get_media_type(".hidden") == "image/png" # "hidden" falls under unknown
    assert get_media_type("multiple.dots.image.jpg") == "image/jpeg"

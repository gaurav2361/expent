import pytest
import sys
import os
from unittest.mock import patch, AsyncMock
from fastapi.testclient import TestClient

# Mock the genai module before importing the main app to avoid API key errors
with patch.dict(os.environ, {"GEMINI_API_KEY": "fake_api_key"}):
    # Add parent directory to path to allow absolute imports
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))
    from apps.ocr.main import app, engine, MAX_FILE_SIZE

client = TestClient(app)


def test_health_check():
    response = client.get("/")
    assert response.status_code == 200
    assert response.json() == {"status": "healthy", "service": "ocr"}


@patch("apps.ocr.main.engine.extract_from_bytes", new_callable=AsyncMock)
def test_extract_success(mock_extract):
    mock_extract.return_value = {"text": "Extracted data", "structured_data": {}}

    # Create a small dummy file
    file_content = b"test file content"
    files = {"file": ("test.txt", file_content, "text/plain")}

    response = client.post("/extract", files=files)

    assert response.status_code == 200
    assert response.json() == {"text": "Extracted data", "structured_data": {}}
    mock_extract.assert_called_once_with(file_content, "test.txt")


@patch("apps.ocr.main.engine.extract_from_bytes", new_callable=AsyncMock)
def test_extract_general_exception(mock_extract):
    mock_extract.side_effect = Exception("Some internal error")

    file_content = b"test file content"
    files = {"file": ("test.txt", file_content, "text/plain")}

    response = client.post("/extract", files=files)

    assert response.status_code == 500
    assert "Extraction failed: Some internal error" in response.json()["detail"]


@patch("apps.ocr.main.engine.extract_from_bytes", new_callable=AsyncMock)
def test_extract_quota_exceeded(mock_extract):
    # Testing 429 quota error logic
    mock_extract.side_effect = Exception("429 Resource has been exhausted (e.g. check quota).")

    file_content = b"test file content"
    files = {"file": ("test.txt", file_content, "text/plain")}

    response = client.post("/extract", files=files)

    assert response.status_code == 429
    assert "Gemini API quota exceeded" in response.json()["detail"]


@patch("apps.ocr.main.engine.extract_from_bytes", new_callable=AsyncMock)
def test_extract_quota_exceeded_word(mock_extract):
    # Testing quota word match
    mock_extract.side_effect = Exception("You have exceeded your QUOTA limits.")

    file_content = b"test file content"
    files = {"file": ("test.txt", file_content, "text/plain")}

    response = client.post("/extract", files=files)

    assert response.status_code == 429
    assert "Gemini API quota exceeded" in response.json()["detail"]


def test_extract_file_too_large():
    # Make a dummy file with large size by mocking the UploadFile reading
    file_content = b"a" * (MAX_FILE_SIZE + 1)
    files = {"file": ("large_file.txt", file_content, "text/plain")}

    response = client.post("/extract", files=files)

    assert response.status_code == 413
    assert "File too large" in response.json()["detail"]


@patch("apps.ocr.main.engine.extract_from_bytes", new_callable=AsyncMock)
def test_ocr_legacy_endpoint(mock_extract):
    mock_extract.return_value = {"text": "Extracted data"}

    file_content = b"test file content"
    files = {"file": ("test.txt", file_content, "text/plain")}

    response = client.post("/ocr", files=files)

    assert response.status_code == 200
    assert response.json() == {"text": "Extracted data"}

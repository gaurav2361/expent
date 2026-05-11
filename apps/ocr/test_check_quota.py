import pytest
import os
import sys
from unittest.mock import patch, MagicMock

# Mock dependencies before importing the code under test
sys.modules["dotenv"] = MagicMock()
sys.modules["google"] = MagicMock()
sys.modules["google.genai"] = MagicMock()

# Add parent directory to path to allow absolute imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))

from apps.ocr.check_quota import check_gemini_quota  # noqa: E402


@patch("apps.ocr.check_quota.load_dotenv")
@patch("apps.ocr.check_quota.os.getenv")
@patch("apps.ocr.check_quota.genai.Client")
def test_check_gemini_quota_success(mock_client_class, mock_getenv, mock_load_dotenv):
    # Setup mocks
    mock_getenv.side_effect = lambda key, default=None: {
        "GOOGLE_API_KEY": "fake_key",
        "GEMINI_MODEL": "gemini-2.5-flash",
    }.get(key, default)

    mock_client = MagicMock()
    mock_client_class.return_value = mock_client
    mock_response = MagicMock()
    mock_response.text = "OK"
    mock_client.models.generate_content.return_value = mock_response

    # Execute
    result = check_gemini_quota()

    # Verify
    assert result is True
    mock_client.models.generate_content.assert_called_once_with(
        model="gemini-2.5-flash", contents="Say 'OK' if you can hear me."
    )


@patch("apps.ocr.check_quota.load_dotenv")
@patch("apps.ocr.check_quota.os.getenv")
def test_check_gemini_quota_no_api_key(mock_getenv, mock_load_dotenv):
    # Setup mock to return None for GOOGLE_API_KEY
    mock_getenv.return_value = None

    # Execute
    result = check_gemini_quota()

    # Verify
    assert result is False


@patch("apps.ocr.check_quota.load_dotenv")
@patch("apps.ocr.check_quota.os.getenv")
@patch("apps.ocr.check_quota.genai.Client")
def test_check_gemini_quota_exceeded(mock_client_class, mock_getenv, mock_load_dotenv):
    # Setup mocks
    mock_getenv.side_effect = lambda key, default=None: {
        "GOOGLE_API_KEY": "fake_key",
        "GEMINI_MODEL": "gemini-2.5-flash",
    }.get(key, default)

    mock_client = MagicMock()
    mock_client_class.return_value = mock_client
    mock_client.models.generate_content.side_effect = Exception("429 Resource has been exhausted (e.g. check quota).")

    # Execute
    result = check_gemini_quota()

    # Verify
    assert result is False


@patch("apps.ocr.check_quota.load_dotenv")
@patch("apps.ocr.check_quota.os.getenv")
@patch("apps.ocr.check_quota.genai.Client")
def test_check_gemini_quota_permission_denied(mock_client_class, mock_getenv, mock_load_dotenv):
    # Setup mocks
    mock_getenv.side_effect = lambda key, default=None: {
        "GOOGLE_API_KEY": "fake_key",
        "GEMINI_MODEL": "gemini-2.5-flash",
    }.get(key, default)

    mock_client = MagicMock()
    mock_client_class.return_value = mock_client
    mock_client.models.generate_content.side_effect = Exception("403 Permission denied.")

    # Execute
    result = check_gemini_quota()

    # Verify
    assert result is False


@patch("apps.ocr.check_quota.load_dotenv")
@patch("apps.ocr.check_quota.os.getenv")
@patch("apps.ocr.check_quota.genai.Client")
def test_check_gemini_quota_unknown_error(mock_client_class, mock_getenv, mock_load_dotenv):
    # Setup mocks
    mock_getenv.side_effect = lambda key, default=None: {
        "GOOGLE_API_KEY": "fake_key",
        "GEMINI_MODEL": "gemini-2.5-flash",
    }.get(key, default)

    mock_client = MagicMock()
    mock_client_class.return_value = mock_client
    mock_client.models.generate_content.side_effect = Exception("Some weird error")

    # Execute
    result = check_gemini_quota()

    # Verify
    assert result is False


@patch("apps.ocr.check_quota.load_dotenv")
@patch("apps.ocr.check_quota.os.getenv")
@patch("apps.ocr.check_quota.genai.Client")
def test_check_gemini_quota_retry_info(mock_client_class, mock_getenv, mock_load_dotenv, capsys):
    # Setup mocks
    mock_getenv.side_effect = lambda key, default=None: {
        "GOOGLE_API_KEY": "fake_key",
        "GEMINI_MODEL": "gemini-2.5-flash",
    }.get(key, default)

    mock_client = MagicMock()
    mock_client_class.return_value = mock_client
    mock_client.models.generate_content.side_effect = Exception("429 Resource exhausted, retry in 15.5s")

    # Execute
    result = check_gemini_quota()

    # Verify
    assert result is False
    captured = capsys.readouterr()
    assert "retry in approximately 15.5 seconds" in captured.out

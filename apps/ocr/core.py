import os
import json
import io
import google.generativeai as genai
import easyocr
from prompts import SYSTEM_PROMPT, USER_PROMPT
from utils import get_media_type, rasterize_pdf_page, extract_pdf_text, parse_csv


class OCREngine:
    def __init__(self, api_key: str = None):
        key = api_key or os.getenv("GOOGLE_API_KEY")
        genai.configure(api_key=key)
        # Gemini 1.5 Flash is retired (as of late 2025/early 2026).
        # Switching to gemini-2.0-flash or gemini-flash-latest.
        self.model_name = "gemini-2.0-flash"
        self.model = genai.GenerativeModel(model_name=self.model_name, system_instruction=SYSTEM_PROMPT)
        self._reader = None
        print(f"DEBUG: OCREngine initialized with model={self.model_name}, key_prefix={key[:8] if key else 'None'}...")

    @property
    def reader(self):
        """Lazy load EasyOCR reader."""
        if self._reader is None:
            # CPU mode by default, can be switched with gpu=True
            self._reader = easyocr.Reader(["en"])
        return self._reader

    async def extract_from_bytes(self, data: bytes, filename: str) -> dict:
        media_type = get_media_type(filename)
        extracted_text = ""
        context_data = None

        if media_type == "text/csv":
            context_data = parse_csv(data)
            extracted_text = json.dumps(context_data)
        elif media_type == "application/pdf":
            # 1. Try text extraction first
            extracted_text = extract_pdf_text(data)
            # 2. If no text found (image-based PDF), Gemini will handle it via its native vision
            # or we could OCR pages here. For now, we'll let Gemini see the PDF.
        elif media_type.startswith("image/"):
            # Use EasyOCR to get raw text as context
            try:
                # EasyOCR expects a numpy array or file path. 
                # For raw bytes, we should use an Image object or check if it's actually an image.
                from PIL import Image
                import numpy as np
                img = Image.open(io.BytesIO(data))
                img_np = np.array(img)
                results = self.reader.readtext(img_np, detail=0)
                extracted_text = " ".join(results)
            except Exception as e:
                print(f"EasyOCR error: {e}")
                extracted_text = ""

        # Gemini logic: using both file and extracted context for better structuring
        content_items = [USER_PROMPT]

        if extracted_text:
            content_items.append(f"EXTRACTED CONTEXT (FROM OCR/PARSER):\n{extracted_text}")

        if media_type == "application/pdf":
            content_items.append({"mime_type": "application/pdf", "data": data})
        elif media_type.startswith("image/"):
            content_items.append({"mime_type": media_type, "data": data})

        # Using structured output mode (JSON Mode)
        try:
            response = self.model.generate_content(
                content_items,
                generation_config=genai.GenerationConfig(response_mime_type="application/json", temperature=0.0),
            )
            return self._parse_json(response.text)
        except Exception as first_err:
            print(f"Primary model ({self.model_name}) failed: {first_err}")
            try:
                # Use the latest stable alias as fallback
                fallback_name = "gemini-flash-latest"
                print(f"Attempting fallback with {fallback_name}...")
                fallback_model = genai.GenerativeModel(model_name=fallback_name, system_instruction=SYSTEM_PROMPT)
                response = fallback_model.generate_content(
                    content_items,
                    generation_config=genai.GenerationConfig(response_mime_type="application/json", temperature=0.0),
                )
                return self._parse_json(response.text)
            except Exception as e:
                # If it's a 429, we should propagate that specifically
                if "429" in str(first_err):
                    raise Exception(f"Quota exceeded (429). Please check your Gemini API billing/limits. {first_err}")
                raise Exception(f"Gemini error: {str(first_err)} -> Fallback error: {str(e)}")

    def _parse_json(self, text: str) -> dict:
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            # Fallback cleanup for occasional hallucinations
            text = text.strip()
            if text.startswith("```"):
                text = text.split("```")[1]
                if text.startswith("json"):
                    text = text[4:]
            return json.loads(text.strip())

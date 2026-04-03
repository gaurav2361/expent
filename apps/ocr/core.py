import os
import json
import io
from google import genai
from google.genai import types
import easyocr
from typing import Dict, Any

from routers.gpay.prompts import GPAY_SYSTEM_PROMPT
from routers.gpay.schemas import GPayExtraction
from routers.generic_receipt.prompts import SYSTEM_PROMPT as GENERIC_SYSTEM_PROMPT, USER_PROMPT as GENERIC_USER_PROMPT
from routers.generic_receipt.schemas import OCRResponse as GenericOCRResponse
from utils import get_media_type, rasterize_pdf_page, extract_pdf_text, parse_csv


class OCREngine:
    def __init__(self, api_key: str = None):
        key = api_key or os.getenv("GOOGLE_API_KEY")
        self.client = genai.Client(api_key=key)
        self.model_name = "gemini-2.0-flash"
        self._reader = None

    @property
    def reader(self):
        """Lazy load EasyOCR reader."""
        if self._reader is None:
            self._reader = easyocr.Reader(["en"])
        return self._reader

    async def classify_image(self, data: bytes, media_type: str) -> str:
        """Classify the image/document type."""
        classification_prompt = "Look at this image. Is it a generic paper retail receipt, an invoice, a bank statement, or a Google Pay digital screenshot? Reply with exactly 'GENERIC' or 'GPAY'."

        try:
            response = self.client.models.generate_content(
                model=self.model_name,
                contents=[
                    classification_prompt,
                    types.Part.from_bytes(data=data, mime_type=media_type)
                ]
            )
            result = response.text.strip().upper()
            if "GPAY" in result:
                return "GPAY"
            return "GENERIC"
        except Exception as e:
            print(f"Classification error: {e}")
            # If it's a quota error, we want to re-raise it so the main app handles it as 429
            if "429" in str(e) or "quota" in str(e).lower():
                raise e
            return "GENERIC"

    async def extract_from_bytes(self, data: bytes, filename: str) -> dict:
        media_type = get_media_type(filename)
        extracted_text = ""

        if media_type.startswith("image/"):
            # Try to get some text context
            try:
                from PIL import Image
                import numpy as np

                img = Image.open(io.BytesIO(data))
                img_np = np.array(img)
                results = self.reader.readtext(img_np, detail=0)
                extracted_text = " ".join(results)
            except Exception as e:
                print(f"EasyOCR error: {e}")

        # Classification
        doc_type = "GENERIC"
        if media_type.startswith("image/"):
            doc_type = await self.classify_image(data, media_type)

        if doc_type == "GPAY":
            system_prompt = GPAY_SYSTEM_PROMPT
            user_prompt = "Extract Google Pay transaction data."
        else:
            system_prompt = GENERIC_SYSTEM_PROMPT
            user_prompt = GENERIC_USER_PROMPT

        content_items = [user_prompt]
        if extracted_text:
            content_items.append(f"EXTRACTED CONTEXT (FROM OCR/PARSER):\n{extracted_text}")

        content_items.append(types.Part.from_bytes(data=data, mime_type=media_type))

        try:
            response = self.client.models.generate_content(
                model=self.model_name,
                contents=content_items,
                config=types.GenerateContentConfig(
                    system_instruction=system_prompt,
                    response_mime_type="application/json",
                    temperature=0.0,
                ),
            )
            return {"doc_type": doc_type, "data": self._parse_json(response.text)}
        except Exception as e:
            print(f"Extraction error: {e}")
            raise e

    def _parse_json(self, text: str) -> dict:
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            text = text.strip()
            if text.startswith("```"):
                text = text.split("```")[1]
                if text.startswith("json"):
                    text = text[4:]
            return json.loads(text.strip())

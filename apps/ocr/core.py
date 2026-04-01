import os
import json
import google.generativeai as genai
from .prompts import SYSTEM_PROMPT, USER_PROMPT
from .utils import get_media_type, rasterize_pdf_page


class OCREngine:
    def __init__(self, api_key: str = None):
        genai.configure(api_key=api_key or os.getenv("GOOGLE_API_KEY"))
        self.model_name = os.getenv("GEMINI_MODEL", "gemini-1.5-flash")
        self.model = genai.GenerativeModel(model_name=self.model_name, system_instruction=SYSTEM_PROMPT)

    async def extract_from_bytes(self, data: bytes, filename: str) -> dict:
        media_type = get_media_type(filename)

        # Gemini 1.5 handles PDF directly or we can rasterize.
        # Direct PDF handling is often better for multi-page.
        content_items = [USER_PROMPT]

        if media_type == "application/pdf":
            # Native PDF support:
            content_items.append({"mime_type": "application/pdf", "data": data})
        else:
            # Image support:
            content_items.append({"mime_type": media_type, "data": data})

        # Using structured output mode (JSON Mode)
        response = self.model.generate_content(
            content_items,
            generation_config=genai.GenerationConfig(response_mime_type="application/json", temperature=0.0),
        )

        return self._parse_json(response.text)

    def _parse_json(self, text: str) -> dict:
        try:
            return json.loads(text)
        except json.JSONDecodeError as e:
            # Fallback cleanup for occasional hallucinations
            text = text.strip()
            if text.startswith("```"):
                text = text.split("```")[1]
                if text.startswith("json"):
                    text = text[4:]
            return json.loads(text.strip())

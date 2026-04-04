from fastapi import FastAPI, UploadFile, HTTPException
from dotenv import load_dotenv
from core import OCREngine

load_dotenv()

app = FastAPI(title="Expent OCR & Data Extraction Service")
engine = OCREngine()

MAX_FILE_SIZE = 20 * 1024 * 1024  # 20MB


@app.get("/")
async def health_check():
    return {"status": "healthy", "service": "ocr"}


@app.post("/extract")
async def extract_data(file: UploadFile):
    """
    Extract structured data from Images (PNG, JPG), PDFs, or CSV files.
    Uses a hybrid approach of traditional OCR/Parsing + Gemini for structuring.
    """
    # Check file size to prevent memory exhaustion (DoS)
    if file.size and file.size > MAX_FILE_SIZE:
        raise HTTPException(
            status_code=413,
            detail=f"File too large. Maximum size allowed is {MAX_FILE_SIZE / 1024 / 1024}MB.",
        )

    try:
        data = await file.read()

        # Re-check actual data size after read in case file.size was missing or incorrect
        if len(data) > MAX_FILE_SIZE:
            raise HTTPException(
                status_code=413,
                detail=f"File too large. Maximum size allowed is {MAX_FILE_SIZE / 1024 / 1024}MB.",
            )

        filename = file.filename or "upload"
        result = await engine.extract_from_bytes(data, filename)
        return result
    except HTTPException:
        raise
    except Exception as e:
        # Check for quota exceeded errors (429)
        error_msg = str(e)
        if "429" in error_msg or "quota" in error_msg.lower():
            raise HTTPException(status_code=429, detail=f"Gemini API quota exceeded: {error_msg}")

        # Log the error in a real app
        raise HTTPException(status_code=500, detail=f"Extraction failed: {error_msg}")


# Keep the old /ocr endpoint for backward compatibility if needed
@app.post("/ocr")
async def ocr_legacy_endpoint(file: UploadFile):
    return await extract_data(file)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8090)

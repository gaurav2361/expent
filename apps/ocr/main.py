from fastapi import FastAPI, UploadFile, HTTPException
from dotenv import load_dotenv
from core import OCREngine

load_dotenv()

app = FastAPI(title="Expent OCR & Data Extraction Service")
engine = OCREngine()


@app.get("/")
async def health_check():
    return {"status": "healthy", "service": "ocr"}


@app.post("/extract")
async def extract_data(file: UploadFile):
    """
    Extract structured data from Images (PNG, JPG), PDFs, or CSV files.
    Uses a hybrid approach of traditional OCR/Parsing + Gemini for structuring.
    """
    try:
        data = await file.read()
        filename = file.filename or "upload"
        result = await engine.extract_from_bytes(data, filename)
        return result
    except Exception as e:
        # Log the error in a real app
        raise HTTPException(status_code=500, detail=f"Extraction failed: {str(e)}")


# Keep the old /ocr endpoint for backward compatibility if needed
@app.post("/ocr")
async def ocr_legacy_endpoint(file: UploadFile):
    return await extract_data(file)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8090)

from fastapi import FastAPI, UploadFile, HTTPException
from dotenv import load_dotenv
from .core import OCREngine

load_dotenv()

app = FastAPI(title="Expent OCR Service")
engine = OCREngine()


@app.post("/ocr")
async def ocr_endpoint(file: UploadFile):
    try:
        data = await file.read()
        filename = file.filename or "upload.png"
        result = await engine.extract_from_bytes(data, filename)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8090)

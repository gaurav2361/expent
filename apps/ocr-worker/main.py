from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import easyocr
import pdfplumber
import boto3
import os
import io
import requests
from typing import Optional, List

app = FastAPI()

# Initialize EasyOCR reader
reader = easyocr.Reader(['en'])

class ProcessRequest(BaseModel):
    url: str
    file_type: str # 'image' or 'pdf'

class TransactionData(BaseModel):
    amount: Optional[float] = None
    date: Optional[str] = None
    upi_id: Optional[str] = None
    contact_name: Optional[str] = None
    raw_text: str

@app.post("/process")
async def process_file(request: ProcessRequest):
    try:
        # 1. Download file from R2
        response = requests.get(request.url)
        if response.status_code != 200:
            raise HTTPException(status_code=400, detail="Failed to download file from R2")
        
        file_content = response.content
        raw_text = ""

        # 2. Extract text
        if request.file_type == 'pdf':
            with pdfplumber.open(io.BytesIO(file_content)) as pdf:
                for page in pdf.pages:
                    raw_text += page.extract_text() or ""
        else:
            # Use EasyOCR for images
            results = reader.readtext(file_content)
            raw_text = " ".join([res[1] for res in results])

        # 3. Basic parsing (To be refined in Phase 4)
        # For now, just return raw text and placeholders
        return {
            "raw_text": raw_text,
            "amount": None, # Logic to extract amount
            "date": None,   # Logic to extract date
            "upi_id": None  # Logic to extract UPI
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8001)

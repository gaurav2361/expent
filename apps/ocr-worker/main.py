from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import easyocr
import pdfplumber
import boto3
import os
import io
import requests
import re
from typing import Optional, List

app = FastAPI()

# Initialize EasyOCR reader
reader = easyocr.Reader(['en'])

class ProcessRequest(BaseModel):
    url: str
    file_type: str # 'image' or 'pdf'

class LineItem(BaseModel):
    name: str
    quantity: int
    price: float

class TransactionData(BaseModel):
    amount: Optional[float] = None
    date: Optional[str] = None
    upi_id: Optional[str] = None
    contact_name: Optional[str] = None
    items: List[LineItem] = []
    raw_text: str

def parse_items(text: str) -> List[LineItem]:
    items = []
    # Very basic regex to find lines like "Item Name 2 x 50.00" or "Item Name 100.00"
    # This is a placeholder for more advanced OCR parsing/NLP
    lines = text.split('\n')
    for line in lines:
        # Try to find a price-like number at the end
        match = re.search(r'(.+?)\s+(\d+\.?\d*)$', line)
        if match:
            name = match.group(1).strip()
            try:
                price = float(match.group(2))
                if price > 0:
                    items.append(LineItem(name=name, quantity=1, price=price))
            except ValueError:
                continue
    return items

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
            raw_text = "\n".join([res[1] for res in results])

        # 3. Basic parsing
        items = parse_items(raw_text)
        
        # Try to find total amount (max price found or specific "TOTAL" keyword)
        amount = None
        total_match = re.search(r'TOTAL\s*:?\s*(\d+\.?\d*)', raw_text, re.IGNORECASE)
        if total_match:
            amount = float(total_match.group(1))
        elif items:
            amount = sum(item.price for item in items)

        return {
            "raw_text": raw_text,
            "amount": amount,
            "date": None, # Date parsing logic can be added here
            "upi_id": None,
            "items": [item.dict() for item in items]
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8001)

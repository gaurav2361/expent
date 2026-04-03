from pydantic import BaseModel, Field
from typing import Optional, List, Any


class LineItem(BaseModel):
    name: str = Field(description="Name of the item")
    quantity: int = Field(default=1)
    price: float = Field(description="Price of the item")


class OCRResponse(BaseModel):
    # Required by Rust OcrResult
    raw_text: str = Field(description="The full raw text extracted from the document")
    amount: Optional[float] = Field(None, description="The total amount of the transaction")
    date: Optional[str] = Field(None, description="The date of the transaction in ISO or readable format")
    upi_id: Optional[str] = Field(None, description="The UPI ID if present")
    items: List[LineItem] = Field(default_factory=list)
    
    # Extra metadata for classification
    document_type: str = Field(description="payment_receipt | invoice | bank_statement | other")
    confidence: str = Field(description="high | medium | low")

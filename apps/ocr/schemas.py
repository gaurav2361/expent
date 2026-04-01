from pydantic import BaseModel
from typing import Optional, List, Any


class LineItem(BaseModel):
    description: str
    quantity: Optional[int] = None
    unit_price: Optional[float] = None
    total: Optional[float] = None


class OCRResponse(BaseModel):
    document_type: str  # "payment_receipt" | "invoice" | "bank_statement" | "other"
    confidence: str  # "high" | "medium" | "low"
    data: Any

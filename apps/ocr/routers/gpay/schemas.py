from pydantic import BaseModel, Field
from typing import Optional, Literal


class GPayExtraction(BaseModel):
    # Core Ledger Data
    amount: float
    direction: Literal["IN", "OUT"]  # "OUT" if "To", "IN" if "From"
    datetime_str: str = Field(description="Exact date and time string e.g. '11 Mar 2026, 1:51 pm'")
    status: Literal["COMPLETED", "PENDING", "FAILED"]

    # Counterparty Info (The person/business you are interacting with)
    counterparty_name: str = Field(description="Name of the person or business")
    counterparty_phone: Optional[str] = None
    counterparty_upi_id: Optional[str] = None
    is_merchant: bool = Field(
        description="True if the counterparty is a business/shop (e.g., has 'vyapar' in UPI or sounds like a store)"
    )

    # Transaction Metadata (Often missing in 'Immediate' screens)
    upi_transaction_id: Optional[str] = None
    google_transaction_id: Optional[str] = None
    source_bank_account: Optional[str] = Field(description="E.g., 'ICICI Bank 0972'")

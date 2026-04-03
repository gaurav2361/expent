### **Current System Analysis and Data Insights**

Your current approach is built primarily to process standard retail or restaurant _receipts_, not _peer-to-peer (P2P) payment confirmations_. Here is the breakdown:

1.  **Code Incompatibility:** Your current Python prompt (`prompts.py`) explicitly looks for a "Vendor" and "Items" (description, quantity, price). The resulting Pydantic schema (`schemas.py`) mirrors this. Your Rust code then extracts only `vendor`, `date`, `amount`, and `payment_method`.
2.  **Image Analysis (What is Actually on the GPay Screen):**
    - **Sending Money (Images 0 and 1):** These screens do not have a "Vendor" (they have a human recipient name: GURJOGESHWAR SINGH), and they absolutely have no "Items" with quantity or price. The key data points are: `Amount`, `Timestamp`, `Recipient Name`, `Google Pay Ref #` (the true transaction ID), and `Payment Method` (e.g., HDFC Bank - 0461).
    - **Receiving Money (Image 2):** This has a _different layout entirely_ from the "sending" screen, using an incoming arrow icon and the large text "Received ₹500 from JATINDER SINGH". The current prompt is completely optimized for this and will likely fail to extract the counterparty or correct amount.
    - **Major Insight:** GPay uses two distinct layouts just for sent vs. received transactions, both of which are radically different from a generic retail receipt. The core issue is trying to force a P2P payment screen into a generic retail receipt schema.

### **The Service-Specific (Google Pay) Plan**

The plan must pivot from a single "Generic Receipt Prompt" to a multi-stage classification and extraction pipeline. I have incorporated your ideas and my suggestions into this reversed plan for the PNG/JPG images provided.

#### **Phase 1: Database Schema Overhaul**

We cannot use the same `Purchase` table/struct for P2P payments as we do for Amazon receipts; the data points are simply too different.

1.  **Introduce `transaction_types` Table:** Add an Enum: `RETAIL_RECEIPT`, `GPay_SENT`, `GPay_RECEIVED`, `PhonePe_SENT`, etc.
2.  **Separate Tables (or Polymorphic Schema):**
    - `purchase` (Existing, for generic receipts: vendor, line_items, total_amount).
    - `p2p_transfer` (New, for P2P apps):
      - `amount`
      - `timestamp`
      - `transaction_type_id`
      - `counterparty_id` (foreign key to a `users` table for internal users, or a text field for external names).
      - `payment_app_id` (Google Pay, PhonePe).
      - `transaction_reference` (The internal ref #, extremely critical for matching).
      - `source_account` (e.g., "HDFC Bank - 0461").

#### **Phase 2: Python OCR Worker Architecture (New Directory Structure)**

Implement your proposed logic for app-specific logic by restructuring the `crates/ocr` directory. This creates isolated modules with their own prompts, schemas, and coordinate-based fallback logic.

```text
crates/ocr/
├── google_pay/
│   ├── prompts.py   <-- Multimodal prompt optimized for both Sent & Received screens
│   ├── schemas.py   <-- Pydantic Schema mapping to 'p2p_transfer' DB table
│   └── parsing.py   <-- Fallback logic (regex/coordinates for GPay date/ref format)
├── generic_receipt/
│   ├── prompts.py   <-- Your current generic receipt prompt
│   └── schemas.py   <-- Your current Pydantic Schema
├── classifier/
│   ├── prompts.py   <-- Super-fast classification prompt
│   └── logic.py
└── core.py          <-- Main entry point, handles classification and routing
```

#### **Phase 3: Classification and Extraction Pipeline**

Update `core.py` to route the image through a classification stage first.

1.  **Stage 1: Classification (The Router):** When an image arrives, the system sends a fast multimodal classification prompt (using Gemini 1.5 Flash) just to ask: _"Is this a Retail Receipt, Google Pay Sent screen, or Google Pay Received screen?"_
2.  **Stage 2: Routing:**
    - If **Retail Receipt:** Use `generic_receipt/prompts.py` and extract to the existing schema.
    - If **Google Pay (Sent or Received):** Route to `google_pay/prompts.py`.
3.  **Stage 3: Service-Specific Extraction:** The dedicated Google Pay prompt (which must include instructions for both "Sent" and "Received" layouts) extracts to the new `GPaySchema` (amount, timestamp, counterparty_name, GPay_ref#, payment_method).
4.  **Stage 4: Validation and Fallback:** The `google_pay/parsing.py` module runs regex/coordinate checks on the raw extracted text as a redundant check to ensure the `Amount` and `Google Pay Ref #` (found at the very bottom) are valid and match.

#### **Phase 5: Rust Client Update**

Update `lib.rs` and your database migration logic to handle the bifurcated output.

1.  **Update Database Migration:** Implement the schema changes discussed in Phase 1 (new tables).
2.  **Update `Purchase` Logic:** The client must now be aware that the OCR service can return either a generic purchase result or a P2P transfer result. It must receive this classified result and route it to the correct DB table (`purchase` vs. `p2p_transfer`).

---

### **The 4 Google Pay Variations We Found:**

1.  **Sent (Detailed View):**
    - **Key Markers:** "To [Name]", Expandable dropdown showing bank details.
    - **Data Richness:** Very High (Amount, Date/Time, Counterparty Name, Counterparty Phone, Counterparty UPI, Your Bank, UPI Txn ID, Google Txn ID).
2.  **Received (Detailed View):**
    - **Key Markers:** "From [Name]".
    - **Data Richness:** Very High (Amount, Date/Time, Sender Name, Sender Bank, Sender UPI, Your Bank, UPI Txn ID).
3.  **Sent to Merchant (Business UPI):**
    - **Key Markers:** The counterparty UPI has "vyapar" (business) or "paytmqr".
    - **Why it matters:** This isn't a P2P split with a friend; this is a pure Retail Expense. Your database needs to treat "LUXMI BIKANER" as a Vendor, not a User Contact.
4.  **Immediate Post-Payment (Quick Success):**
    - **Key Markers:** Big blue checkmark, "Paid to [Name]", "Banking name: [Name]".
    - **Data Richness:** Very Low. Notice what's missing: **There is no UPI Transaction ID and no Bank info here.**

---

### **The Updated Python OCR Architecture Plan**

Because the data richness varies wildly (Variation 1 has everything, Variation 4 has very little), our Pydantic schemas must use `Optional` heavily, and our prompt needs to be incredibly smart about establishing the **Direction** of the money.

Here is the concrete plan for your Python worker.

#### **1. The New Directory Structure**

Create a highly modular structure inside your OCR app so you can easily add PhonePe or Paytm later.

```text
apps/ocr/
├── main.py
├── core.py               # The main router (Gemini Vision classifier)
├── routers/
│   ├── gpay/
│   │   ├── prompts.py    # The GPay-specific prompt
│   │   └── schemas.py    # The GPay-specific Pydantic models
│   ├── generic_receipt/
│   │   ├── prompts.py
│   │   └── schemas.py
```

#### **2. The GPay Schema (`routers/gpay/schemas.py`)**

We will create a unified schema that handles all four variations. The `direction` field is the most critical addition here, directly mapping to your Rust database's `TransactionDirection` enum (`IN` or `OUT`).

```python
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
    is_merchant: bool = Field(description="True if the counterparty is a business/shop (e.g., has 'vyapar' in UPI or sounds like a store)")

    # Transaction Metadata (Often missing in 'Immediate' screens)
    upi_transaction_id: Optional[str] = None
    google_transaction_id: Optional[str] = None
    source_bank_account: Optional[str] = Field(description="E.g., 'ICICI Bank 0972'")
```

#### **3. The GPay Prompt (`routers/gpay/prompts.py`)**

Instead of the generic prompt you had, we will give Gemini explicit instructions on how to read the GPay UI.

```python
GPAY_SYSTEM_PROMPT = """
You are a highly precise financial data extractor analyzing Google Pay screenshots.
Your goal is to extract transaction details into the provided JSON schema.

CRITICAL RULES FOR GOOGLE PAY:
1. Direction:
   - If the screen says "To [Name]" or "Paid to [Name]", the direction is "OUT".
   - If the screen says "From [Name]", the direction is "IN".
2. Counterparty:
   - Extract the primary name shown (e.g., "GURJOGESHWAR Singh" or "LUXMI BIKANER MISTHAN BHANDAR").
   - Extract their phone number (+91...) if visible under their name.
3. Merchant Detection:
   - Set "is_merchant" to true IF the name implies a business (e.g., "Misthan Bhandar", "Store", "Cafe") OR if their UPI ID contains "vyapar", "paytmqr", or "merchant".
4. Metadata:
   - Look at the bottom details box for "UPI transaction ID" and "Google transaction ID".
   - If this is a simple blue checkmark screen, these IDs will not be visible. Set them to null. Do NOT guess.
5. Formatting:
   - Strip the currency symbol (₹) and commas from the amount. Return a pure float.
"""
```

#### **4. The Router Logic (`core.py`)**

Update your `extract_from_bytes` method to first classify the image, then route it to the specific schema.

```python
# Pseudo-code for core.py logic

# Step 1: Fast Classification (Using gemini-2.5-flash)
classification_prompt = "Look at this image. Is it a generic paper retail receipt, or a Google Pay digital screenshot? Reply with exactly 'GENERIC' or 'GPAY'."
classification = await model.generate_content([classification_prompt, image_data])

# Step 2: Route to specific schema
if "GPAY" in classification.text:
    response = model.generate_content(
        [GPAY_SYSTEM_PROMPT, image_data],
        generation_config=genai.GenerationConfig(
            response_mime_type="application/json",
            response_schema=GPayExtraction # Use Pydantic schema here!
        )
    )
    return response.text
```

### **How This Helps Your Rust Backend (`lib.rs`)**

Once the Python worker returns this specific JSON, your Rust backend's `SmartMerge` logic becomes incredibly powerful:

1.  **It knows the direction:** `direction: "OUT"` immediately tells SeaORM to debit your account.
2.  **It separates merchants from friends:** If `is_merchant: true` (like the Bikaner shop), Rust can skip creating a `p2p_request` or looking up a User Contact, and instead just log it as a standard `Purchase` expense.
3.  **It handles missing data:** Because `upi_transaction_id` is optional, if the user uploads the "Immediate Blue Checkmark" screen, the app accepts it. If the user later uploads their bank statement CSV, `SmartMerge` can match the statement to this transaction using just the `amount` and `datetime_str`.

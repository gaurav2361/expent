FROM python:3.13-slim

WORKDIR /app

# Install system dependencies required for OpenCV and PDF parsing
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    libgl1 \
    libglib2.0-0 \
    && rm -rf /var/lib/apt/lists/*

# Install uv package manager
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"

# Install Python dependencies globally for Docker using uv
# Note: google-genai and easyocr are heavy, caching this layer is important
RUN --mount=type=cache,target=/root/.cache/uv \
    uv pip install --system \
    fastapi \
    uvicorn \
    easyocr \
    pdfplumber \
    requests \
    python-multipart \
    python-dotenv \
    pymupdf \
    google-genai

# Copy only the OCR source to keep it lean
COPY apps/ocr ./apps/ocr

# Shift workdir to OCR app
WORKDIR /app/apps/ocr

EXPOSE 8090
ENV PYTHONUNBUFFERED=1

# Use JSON form for CMD
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8090"]

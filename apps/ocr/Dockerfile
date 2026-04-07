FROM python:3.13-slim

# Use the official uv image as a source for the uv binary
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

WORKDIR /app

# Install system dependencies required for OpenCV and PDF parsing
RUN apt-get update && apt-get install -y \
    build-essential \
    libgl1 \
    libglib2.0-0 \
    && rm -rf /var/lib/apt/lists/*

# Use uv for faster dependency installation and better caching
# 1. First, only copy files required for dependency installation to leverage Docker cache
COPY pyproject.toml uv.lock ./

# 2. Install dependencies (creates a .venv in /app)
# We use a cache mount for the uv cache to speed up subsequent builds
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --frozen --no-dev

# 3. Copy the OCR app source code
COPY apps/ocr ./apps/ocr

# Shift workdir to OCR app
WORKDIR /app/apps/ocr

# Place the virtual environment's bin at the front of the PATH
ENV PATH="/app/.venv/bin:$PATH"
EXPOSE 8090
ENV PYTHONUNBUFFERED=1

# Run the app directly using the environment's uvicorn
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8090"]

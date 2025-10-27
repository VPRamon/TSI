# syntax=docker/dockerfile:1.6
FROM python:3.11-slim AS base

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1

WORKDIR /app

# System dependencies required for many python scientific packages
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        git \
    && rm -rf /var/lib/apt/lists/*

# Copy project files needed for installation
COPY pyproject.toml ./
COPY requirements.txt ./
COPY README.md ./
COPY src ./src

# Install python dependencies (includes -e . from requirements.txt)
RUN pip install --upgrade pip \
    && pip install -r requirements.txt

# Copy the rest of the project files
COPY . .

EXPOSE 8501

# By default run the Streamlit dashboard, but allow overriding the command for tests
CMD ["streamlit", "run", "src/tsi/app.py", "--server.address=0.0.0.0", "--server.port=8501"]

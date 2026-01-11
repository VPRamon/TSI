# syntax=docker/dockerfile:1

FROM python:3.11-slim AS base
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1
WORKDIR /app


FROM base AS dev
RUN apt-get update && apt-get install -y --no-install-recommends \
        bash \
        build-essential \
        ca-certificates \
        curl \
        git \
        libpq-dev \
        patchelf \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash app

RUN python -m venv /opt/venv
RUN chown -R app:app /opt/venv
ENV VIRTUAL_ENV=/opt/venv
ENV PATH="/opt/venv/bin:${PATH}"
RUN printf '%s\n' \
    'export VIRTUAL_ENV=/opt/venv' \
    'export PATH="$VIRTUAL_ENV/bin:$PATH"' \
    > /etc/profile.d/venv.sh
RUN pip install --no-cache-dir --upgrade pip && pip install --no-cache-dir "maturin>=1.0,<2.0"
RUN pip install --no-cache-dir ruff black mypy pandas-stubs types-PyYAML types-Markdown

USER app
ENV RUSTUP_HOME=/home/app/.rustup \
    CARGO_HOME=/home/app/.cargo
ENV PATH="/opt/venv/bin:/home/app/.cargo/bin:${PATH}"
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
WORKDIR /workspace
ENV PYTHONPATH=/workspace/src


FROM base AS rust-wheel-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        patchelf \
        pkg-config \
        libpq-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

RUN pip install --no-cache-dir "maturin>=1.0,<2.0"

COPY backend/ ./backend/
RUN maturin build \
        --release \
        --manifest-path backend/Cargo.toml \
        --features postgres-repo \
        -o /tmp/wheels


FROM base AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
        libpq5 \
        libgomp1 \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.base.txt ./requirements.base.txt
RUN pip install --no-cache-dir -r requirements.base.txt

COPY --from=rust-wheel-builder /tmp/wheels /tmp/wheels
RUN pip install --no-cache-dir /tmp/wheels/*.whl && rm -rf /tmp/wheels

COPY src/ ./src/
COPY data/ ./data/
COPY .streamlit/ ./.streamlit/

ENV PYTHONPATH=/app/src
EXPOSE 8501

CMD ["streamlit", "run", "src/tsi/app.py", "--server.address=0.0.0.0", "--server.port=8501"]

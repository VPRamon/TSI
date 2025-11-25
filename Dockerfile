# syntax=docker/dockerfile:1.6

ARG DEBIAN_VERSION=12
ARG RUST_VERSION=1.74.1
ARG APP_USER=app

#############################
# Base image with Python runtime
#############################
FROM debian:${DEBIAN_VERSION}-slim AS base

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        python3 \
        python3-venv \
        python3-pip \
        python3-dev \
        tini \
    && rm -rf /var/lib/apt/lists/*

#############################
# Build dependencies (C toolchain, git, curl)
#############################
FROM base AS build-deps
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        libffi-dev \
        liblzma-dev \
        libbz2-dev \
        libreadline-dev \
        libxml2-dev \
        libxslt1-dev \
        git \
        curl \
        unzip \
    && rm -rf /var/lib/apt/lists/*

#############################
# Rust toolchain + cargo-chef for dependency caching
#############################
FROM build-deps AS rust-base
ARG RUST_VERSION
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal \
    && rustup target add x86_64-unknown-linux-gnu \
    && cargo install cargo-chef --locked

#############################
# Cargo Chef planner (captures dependency graph for caching)
#############################
FROM rust-base AS cargo-planner
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY rust_backend/Cargo.toml rust_backend/Cargo.toml
COPY rust_backend/src rust_backend/src
COPY rust_backend/siderust rust_backend/siderust
COPY rust_backend/tests rust_backend/tests
RUN cargo chef prepare --recipe-path recipe.json

#############################
# Cargo Chef cook + Rust build (produces wheels via maturin)
#############################
FROM rust-base AS cargo-builder
WORKDIR /workspace
COPY --from=cargo-planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY pyproject.toml README.md ./
COPY requirements.base.txt requirements.base.txt
COPY rust_backend rust_backend

RUN python3 -m pip install --upgrade pip \
    && pip install --no-cache-dir maturin==1.6.0
RUN maturin build \
        --manifest-path rust_backend/Cargo.toml \
        --release \
        --locked \
        --out /artifacts \
        --skip-auditwheel \
        --no-sdist

#############################
# Python dependency builder (creates reusable virtualenv)
#############################
FROM build-deps AS python-builder
ENV VIRTUAL_ENV=/opt/venv \
    PATH=/opt/venv/bin:$PATH
RUN python3 -m venv ${VIRTUAL_ENV}
WORKDIR /workspace
COPY requirements.base.txt ./
RUN pip install --upgrade pip wheel \
    && pip install --no-cache-dir -r requirements.base.txt
COPY --from=cargo-builder /artifacts /tmp/artifacts
RUN pip install --no-cache-dir /tmp/artifacts/*.whl \
    && rm -rf /tmp/artifacts

#############################
# Production runtime image
#############################
FROM base AS runtime
ENV PATH=/opt/venv/bin:$PATH \
    PYTHONPATH=/app/src \
    STREAMLIT_SERVER_ADDRESS=0.0.0.0 \
    STREAMLIT_SERVER_PORT=8501
WORKDIR /app
COPY --from=python-builder /opt/venv /opt/venv
COPY src ./src
COPY data ./data
COPY streamlit_app.py ./streamlit_app.py
COPY run_dashboard.sh ./run_dashboard.sh
COPY README.md pyproject.toml requirements.base.txt ./
COPY .streamlit ./.streamlit

EXPOSE 8501
ENTRYPOINT ["tini", "--"]
CMD ["streamlit", "run", "src/tsi/app.py", "--server.address=0.0.0.0", "--server.port=8501"]

#############################
# Developer shell image with toolchains + dev deps
#############################
FROM rust-base AS dev
ENV VIRTUAL_ENV=/opt/venv \
    PATH=/opt/venv/bin:/opt/cargo/bin:$PATH \
    PYTHONPATH=/workspace/src
COPY --from=python-builder /opt/venv /opt/venv
WORKDIR /workspace
COPY . .
RUN groupadd --gid 1000 ${APP_USER} \
    && useradd --uid 1000 --gid ${APP_USER} --shell /bin/bash --create-home ${APP_USER} \
    && chown -R ${APP_USER}:${APP_USER} /workspace /opt/venv /opt/cargo /opt/rustup
USER ${APP_USER}
RUN pip install --upgrade pip \
    && pip install --no-cache-dir -e ".[dev]"
CMD ["bash"]

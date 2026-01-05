# syntax=docker/dockerfile:1.6

ARG DEBIAN_VERSION=12
ARG RUST_VERSION=nightly
ARG APP_USER=app

#############################
# Minimal runtime base (no dev packages)
#############################
FROM debian:${DEBIAN_VERSION}-slim AS runtime-base

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
        tini \
    && rm -rf /var/lib/apt/lists/*

#############################
# Build base with development packages
#############################
FROM runtime-base AS build-base

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        python3-dev \
    && rm -rf /var/lib/apt/lists/*

#############################
# Build dependencies (C toolchain, git, curl)
#############################
FROM build-base AS build-deps
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
        unixodbc \
        unixodbc-dev \
        gnupg \
    && curl -sSL https://packages.microsoft.com/keys/microsoft.asc \
        | gpg --dearmor \
        | tee /usr/share/keyrings/microsoft-prod.gpg >/dev/null \
    && echo "deb [arch=amd64,arm64,armhf signed-by=/usr/share/keyrings/microsoft-prod.gpg] https://packages.microsoft.com/debian/12/prod bookworm main" \
        | tee /etc/apt/sources.list.d/mssql-release.list >/dev/null \
    && apt-get update \
    && ACCEPT_EULA=Y apt-get install -y msodbcsql18 \
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
    && rustup component add rustfmt clippy \
    && cargo install cargo-chef --locked

#############################
# Cargo Chef planner (captures dependency graph for caching)
#############################
FROM rust-base AS cargo-planner
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY rust_backend/Cargo.toml rust_backend/Cargo.toml
# Ensure local path crates referenced by the workspace are present for cargo metadata
COPY rust_backend/qtty rust_backend/qtty
# Create dummy lib.rs to satisfy cargo metadata
RUN mkdir -p rust_backend/src && echo "// dummy" > rust_backend/src/lib.rs
RUN cargo chef prepare --recipe-path recipe.json

#############################
# Cargo Chef cook + Rust build (produces wheels via maturin)
#############################
FROM rust-base AS cargo-builder

# Repository backend feature: local-repo (default), postgres-repo (production), azure-repo
ARG REPO_FEATURE=postgres-repo

WORKDIR /workspace
COPY --from=cargo-planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY pyproject.toml README.md ./
COPY requirements.base.txt requirements.base.txt
COPY rust_backend rust_backend

RUN python3 -m pip install --upgrade pip --break-system-packages \
    && pip install --no-cache-dir maturin==1.6.0 --break-system-packages
RUN maturin build \
        --manifest-path rust_backend/Cargo.toml \
        --release \
        --locked \
        --out /artifacts \
        --skip-auditwheel \
        --no-default-features \
        --features ${REPO_FEATURE}

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
    && rm -rf /tmp/artifacts \
    && find /opt/venv -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true \
    && find /opt/venv -type f -name "*.pyc" -delete \
    && find /opt/venv -type f -name "*.pyo" -delete

#############################
# Production runtime image
#############################
FROM runtime-base AS runtime
ARG APP_USER=app

ENV PATH=/opt/venv/bin:$PATH \
    PYTHONPATH=/app/src \
    STREAMLIT_SERVER_ADDRESS=0.0.0.0 \
    STREAMLIT_SERVER_PORT=8501

# Create non-root user
RUN groupadd --gid 1000 ${APP_USER} \
    && useradd --uid 1000 --gid ${APP_USER} --shell /bin/bash --create-home ${APP_USER}

WORKDIR /app

COPY --from=python-builder /opt/venv /opt/venv
COPY src ./src
COPY data ./data
COPY streamlit_app.py ./streamlit_app.py
COPY run_dashboard.sh ./run_dashboard.sh
COPY README.md pyproject.toml requirements.base.txt ./
COPY .streamlit ./.streamlit

# Set ownership to non-root user
RUN chown -R ${APP_USER}:${APP_USER} /app /opt/venv

USER ${APP_USER}

EXPOSE 8501
ENTRYPOINT ["tini", "--"]
CMD ["streamlit", "run", "src/tsi/app.py", "--server.address=0.0.0.0", "--server.port=8501"]

#############################
# Developer shell image with toolchains + dev deps
#############################
FROM rust-base AS dev
ARG APP_USER=app
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
# Ensure PATH is set in login shells
RUN echo 'export PATH=/opt/venv/bin:/opt/cargo/bin:$PATH' >> ~/.bashrc \
    && echo 'export PATH=/opt/venv/bin:/opt/cargo/bin:$PATH' >> ~/.bash_profile \
    && echo 'export PYTHONPATH=/workspace/src' >> ~/.bashrc \
    && echo 'export PYTHONPATH=/workspace/src' >> ~/.bash_profile
RUN pip install --upgrade pip \
    && pip install --no-cache-dir \
        pytest>=7.4.0 \
        pytest-cov>=4.1.0 \
        hypothesis>=6.98.0 \
        responses>=0.25.0 \
        ruff>=0.1.0 \
        black>=23.11.0 \
        mypy>=1.7.0 \
        pandas-stubs>=2.0.0 \
        types-PyYAML>=6.0.0 \
        types-Markdown>=3.0.0
CMD ["bash"]

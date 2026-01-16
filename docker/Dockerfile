# syntax=docker/dockerfile:1

FROM python:3.11-slim AS base
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1
WORKDIR /app


FROM base AS rust-wheel-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
    cmake \
        curl \
    git \
    libboost-all-dev \
    libnova-dev \
    nlohmann-json3-dev \
    libxml++2.6-dev \
    libxml2-dev \
        patchelf \
        pkg-config \
        libpq-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

RUN pip install --no-cache-dir "maturin>=1.0,<2.0"

RUN rustup component add rustfmt clippy || true

COPY backend/ ./backend/
RUN maturin build \
        --release \
        --manifest-path backend/Cargo.toml \
        --features postgres-repo \
        -o /tmp/wheels


FROM base AS dev
RUN apt-get update && apt-get install -y --no-install-recommends \
        bash \
        build-essential \
        ca-certificates \
    cmake \
        curl \
        git \
    libboost-all-dev \
    libnova-dev \
    nlohmann-json3-dev \
    libxml++2.6-dev \
    libxml2-dev \
        libpq-dev \
        patchelf \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain in a shared location so CI can run the container as an
# arbitrary UID/GID (e.g. GitHub Actions runner) without relying on a user home.
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo
RUN set -eux; \
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable; \
    /opt/cargo/bin/rustup component add rustfmt clippy; \
    toolchain_bin_dir="$(dirname "$(/opt/cargo/bin/rustup which cargo)")"; \
    echo "export PATH=${toolchain_bin_dir}:\$PATH" > /etc/profile.d/rust-path.sh; \
    chmod 0644 /etc/profile.d/rust-path.sh; \
    for tool in cargo rustc rustdoc rustfmt clippy-driver cargo-clippy cargo-fmt; do \
        tool_path="$(/opt/cargo/bin/rustup which "${tool}" || true)"; \
        if [ -n "${tool_path}" ] && [ -x "${tool_path}" ]; then \
            ln -sf "${tool_path}" "/usr/local/bin/${tool}"; \
        fi; \
    done

# Install Python dependencies globally (no venv) to avoid mixing environments.
COPY requirements.base.txt ./requirements.base.txt
RUN pip install --no-cache-dir --upgrade pip \
    && pip install --no-cache-dir -r requirements.base.txt \
    && pip install --no-cache-dir \
        pytest \
        "maturin>=1.0,<2.0" \
        ruff \
        black \
        mypy \
        pandas-stubs \
        types-PyYAML \
        types-Markdown

# Install the Rust extension wheel so imports work in the dev container.
COPY --from=rust-wheel-builder /tmp/wheels /tmp/wheels
RUN pip install --no-cache-dir /tmp/wheels/*.whl && rm -rf /tmp/wheels

RUN useradd -m -s /bin/bash app

USER app
WORKDIR /workspace
ENV PYTHONPATH=/workspace/src

# Keep rustup/cargo homes for interactive usage, but the shared toolchain above
# is what CI relies on when running as an arbitrary UID/GID.
ENV RUSTUP_HOME=/home/app/.rustup \
    CARGO_HOME=/home/app/.cargo
ENV PATH="/home/app/.cargo/bin:${PATH}"


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

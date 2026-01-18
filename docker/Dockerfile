# Dev Dockerfile for VS Code Dev Container
# Provides a lightweight development image with Python and Rust toolchain

FROM python:3.11-slim AS base

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    ca-certificates \
    git \
    gcc \
    libssl-dev \
    pkg-config \
    cmake \
    # Node.js & npm for frontend development inside the dev container
    nodejs \
    npm \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a shared virtualenv for VS Code to use
RUN python -m venv /opt/venv && /opt/venv/bin/python -m pip install --upgrade pip setuptools wheel

# Dev stage used by docker-compose.devcontainer.yml
FROM base AS dev

# Create non-root user 'app' with home directory
ARG USER=app
ARG UID=1000
ARG GID=1000
RUN groupadd -g ${GID} ${USER} || true \
    && useradd -m -u ${UID} -g ${GID} -s /bin/bash ${USER}

# Create workspace and point permissions to the app user
RUN mkdir -p /workspace && chown -R ${USER}:${USER} /workspace /opt/venv

# Install Rust toolchain for the non-root dev user and add useful components
USER ${USER}
WORKDIR /workspace

ENV PATH="/home/${USER}/.cargo/bin:/opt/venv/bin:${PATH}"
ENV PYTHONPATH="/workspace/src"

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path \
    && ~/.cargo/bin/rustup default stable || true \
    && ~/.cargo/bin/rustup component add rustfmt clippy || true

CMD ["sleep", "infinity"]

# Expose frontend dev port so host/container tooling can map it
EXPOSE 3000

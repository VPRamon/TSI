FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN apt-get update && \
    apt-get install -y libclang-dev build-essential pkg-config && \
    rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tsi_backend /app/tsi_backend
EXPOSE 8081
ENV RUST_LOG=info
CMD ["/app/tsi_backend"]

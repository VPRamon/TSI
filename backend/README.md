# TSI Backend (Rust + Axum)

Production-grade REST API for telescope scheduling analytics.

## Features

- **Health check**: `GET /health`
- **Compute endpoint**: `POST /api/v1/compute` (accepts `{ values: number[] }`, returns `{ mean, std }`)
- **SSE progress**: `GET /api/v1/progress` (demo of streaming progress updates)

## Running Locally

```bash
cd backend
cargo run
```

Server listens on `http://127.0.0.1:8080`.

## Testing

```bash
cargo test
```

Integration tests are marked `#[ignore]` by default (require a running server).

## Docker

```bash
docker build -t tsi-backend .
docker run -p 8080:8080 tsi-backend
```

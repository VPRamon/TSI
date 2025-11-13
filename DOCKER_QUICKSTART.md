# Docker Quick Start Guide

## Starting the Rust Backend

### Option 1: Using the startup script
```bash
./start-rust-backend.sh
```

### Option 2: Using Docker Compose directly
```bash
# Start only the Rust backend
docker-compose up --build rust-backend

# Or run in detached mode (background)
docker-compose up -d --build rust-backend
```

### Option 3: Start everything (backend + frontend)
```bash
docker-compose up --build
```

## Accessing the Services

- **Rust Backend API**: http://localhost:8081
- **Health Check**: http://localhost:8081/health
- **API Documentation**: http://localhost:8081/swagger-ui (if configured)
- **Frontend** (if started): http://localhost:5173

## Useful Commands

```bash
# Stop all services
docker-compose down

# View logs
docker-compose logs -f rust-backend

# Rebuild without cache
docker-compose build --no-cache rust-backend

# Stop and remove volumes
docker-compose down -v
```

## Configuration

The backend is configured with:
- **Port**: 8081
- **Log Level**: Info (set via `RUST_LOG` environment variable)
- **Data Volume**: `./data` mounted as read-only at `/app/data`
- **Network**: `tsi-network` bridge network

## Troubleshooting

If you encounter issues:

1. **Port already in use**: Make sure port 8081 is not being used by another application
2. **Build failures**: Try `docker-compose build --no-cache rust-backend`
3. **Check logs**: `docker-compose logs rust-backend`

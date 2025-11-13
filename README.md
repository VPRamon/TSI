# Telescope Scheduling Intelligence

Modern web application for analyzing and visualizing astronomical scheduling outputs with a high-performance Rust backend and Vue.js frontend.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Vue](https://img.shields.io/badge/vue-3.0%2B-green.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)

## Architecture

TSI is built with a modern microservices architecture:

- **Backend**: Rust + Axum + Polars (high-performance REST API)
- **Frontend**: Vue 3 + TypeScript + Vite (reactive SPA)
- **Data**: JSON/CSV schedule files with preprocessing pipeline
- **Deployment**: Docker + Docker Compose

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for detailed architecture diagrams and data flow.

## Quick Start

### Using Docker Compose (Recommended)

Start both backend and frontend with a single command:

\`\`\`bash
docker-compose up --build
\`\`\`

Access the application:
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:8081
- **Health Check**: http://localhost:8081/health

See [`DOCKER_QUICKSTART.md`](DOCKER_QUICKSTART.md) for more Docker commands.

### Manual Setup

#### Backend (Rust)

\`\`\`bash
cd backend
cargo run
\`\`\`

Server runs at http://127.0.0.1:8080. See [`backend/README.md`](backend/README.md) for details.

#### Frontend (Vue)

\`\`\`bash
cd frontend
npm install
npm run dev
\`\`\`

Frontend runs at http://localhost:5173. See [`frontend/README.md`](frontend/README.md) for details.

## Repository Structure

\`\`\`
.
├── backend/              # Rust API server (Axum + Polars)
│   ├── src/
│   │   ├── main.rs      # Server entrypoint
│   │   ├── routes.rs    # HTTP handlers
│   │   ├── compute.rs   # Analytics logic
│   │   └── models/      # Data models
│   ├── tests/           # Integration tests
│   └── benches/         # Performance benchmarks
│
├── frontend/            # Vue 3 + TypeScript SPA
│   ├── src/
│   │   ├── App.vue      # Main application
│   │   ├── components/  # Vue components
│   │   └── pages/       # Page views
│   └── package.json
│
├── data/                # Sample datasets
│   ├── schedule.csv
│   ├── schedule.json
│   └── dark_periods.json
│
├── src/                 # Python utilities (legacy preprocessing)
│   ├── core/           # Core preprocessing library
│   │   ├── loaders/    # Data loaders
│   │   └── preprocessing/
│   └── adapters/       # Data adapters
│
├── tests/              # Python core library tests
│   └── core/
│
├── docs/               # Documentation
│   ├── API.md
│   └── *.md
│
├── migration-doc/      # Migration documentation
│   └── PHASE_*.md
│
├── old/               # Legacy application (archived)
│   └── ...
│
├── docker-compose.yml
├── ARCHITECTURE.md
└── README.md
\`\`\`

## Features

### Current Capabilities

- **Health Monitoring**: System health checks via REST API
- **Data Analysis**: Statistical computations (mean, std dev) on datasets
- **Real-time Updates**: Server-Sent Events (SSE) for progress tracking
- **Modern UI**: Responsive Vue 3 interface
- **Performance**: Rust backend with Polars for fast data processing

### API Endpoints

- \`GET /health\` - Health check
- \`POST /api/v1/compute\` - Compute statistics on input data
- \`GET /api/v1/progress\` - SSE stream for progress updates

## Data Format

Schedule data can be provided as:
- **JSON**: Raw schedule format (see \`data/schedule.json\`)
- **CSV**: Preprocessed format for faster loading

Required columns include:
- \`schedulingBlockId\`, \`priority\`, \`requestedDurationSec\`
- \`fixedStartTime\`, \`fixedStopTime\`
- \`raInDeg\`, \`decInDeg\`
- \`minElevationAngleInDeg\`, \`maxElevationAngleInDeg\`

## Configuration

### Backend Environment Variables

- \`RUST_LOG\`: Log level (info, debug, trace)
- \`RUST_BACKTRACE\`: Enable backtraces (0, 1, full)

### Frontend Configuration

Edit \`frontend/vite.config.ts\` to configure API proxy and build settings.

## Development

### Backend Testing

\`\`\`bash
cd backend
cargo test              # Run all tests
cargo test -- --ignored # Run integration tests
cargo bench            # Run benchmarks
\`\`\`

### Frontend Testing

\`\`\`bash
cd frontend
npm test               # Run tests
npm run build         # Production build
\`\`\`

### Python Core Library

The \`src/core/\` directory contains Python utilities for data preprocessing (legacy support):

\`\`\`bash
# Run core library tests
pytest tests/core/
\`\`\`

## Migration History

This project has been migrated from a Python/Streamlit application to a modern Rust/Vue stack. The legacy application and all related files are preserved in the \`/old\` directory for reference.

Migration documentation can be found in:
- [\`migration-doc/RUST_VUE_MIGRATION_PLAN.md\`](migration-doc/RUST_VUE_MIGRATION_PLAN.md)
- [\`migration-doc/PHASE_*.md\`](migration-doc/) - Detailed phase-by-phase migration logs

## Documentation

- [\`ARCHITECTURE.md\`](ARCHITECTURE.md) - System architecture and design
- [\`DOCKER_QUICKSTART.md\`](DOCKER_QUICKSTART.md) - Docker deployment guide
- [\`docs/API.md\`](docs/API.md) - API documentation
- [\`backend/README.md\`](backend/README.md) - Backend development guide
- [\`frontend/README.md\`](frontend/README.md) - Frontend development guide

## License

AGPL-3.0 — see [\`LICENSE\`](LICENSE) for details.

---

**Built with**: Rust, Axum, Polars, Vue 3, TypeScript, Docker

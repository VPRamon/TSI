Here is the **English translation** of the report, preserving the technical tone, structure, and intent for architectural and stakeholder discussions.

---

# Technical Report (Markdown) — Migration to a Client–Server Architecture Based on the Current Codebase

## 1) Executive Summary

The repository already contains a **functional** Rust “backend” (local or PostgreSQL persistence, service layer, serializable DTOs, tests), but it is **embedded** inside the Streamlit application via **PyO3/maturin bindings** (Python module `tsi_rust`).
The proposed migration consists of **promoting** this Rust backend to an **HTTP service** (axum) and replacing Streamlit with a **web frontend** (React + TypeScript), while maximizing reuse of the existing code.

Expected outcome:

* Rust backend accessible over the network (REST/JSON + SSE for progress).
* Decoupled frontend (web client) with interactive, browser-rendered plots.
* Configurable persistence: `LocalRepository` (dev/ephemeral) or `PostgresRepository` (prod).
* Foundation ready for concurrent multi-client usage (no authentication at this stage).

---

## 2) Current State (AS-IS) Based on the Repository

### 2.1 Current Frontend (UI + orchestration monolith)

* Streamlit lives under `src/tsi/`:

  * Entrypoint: `src/tsi/app.py`
  * Pages: `src/tsi/pages/*` (Sky Map, Distributions, Visibility Map, Timeline, Insights, Trends, Compare, Validation)
  * Session state: `src/tsi/state.py` (uses `tsi_rust.ScheduleId` as dataset reference)
* The UI delegates to a Python “facade”:

  * `src/tsi/services/backend_service.py` encapsulates calls to Rust (`tsi_rust`) and exposes methods such as `upload_schedule`, `list_schedules`, `get_*_data`.

### 2.2 Current Backend (Rust as a library + Python bindings)

* Rust crate located at `backend/` (`backend/Cargo.toml`) compiled as:

  * `cdylib` + `rlib` (`[lib] crate-type = ["cdylib", "rlib"]`)
* Python exposure:

  * `pyproject.toml` uses `maturin` with `bindings = "pyo3"` and `module-name = "tsi_rust"`
  * PyO3 modules defined in `backend/src/lib.rs` (`#[pymodule] fn tsi_rust(...)` and `tsi_rust_api(...)`)
* Internal Rust organization (already aligned with a “backend” architecture):

  * `backend/src/models/*`: parsing and domain models (e.g., schedule JSON parsing)
  * `backend/src/db/*`: repository + factory + services:

    * Repository pattern with traits (schedule / analytics / validation / visualization)
    * Backend selection via feature/env (`postgres-repo` vs `local-repo`)
    * Global initialization using `OnceLock` (`backend/src/db/mod.rs`)
  * `backend/src/services/*`: use cases and queries for visualizations (sky map, insights, trends, compare, etc.)
  * `backend/src/routes/*`: *conceptual* “routes” (non-HTTP) that register functions/constants for Python:

    * Example: `backend/src/routes/landing.rs` exposes `store_schedule`, `list_schedules`, and constants `POST_SCHEDULE`, `LIST_SCHEDULES`
    * DTOs already derive `serde::{Serialize, Deserialize}` in several modules (e.g., `SkyMapData`, `ScheduleInfo`, etc.)

### 2.3 Persistence and Local Execution (Already Present)

* Repository support:

  * `LocalRepository` (in-memory, ephemeral) and `PostgresRepository` (Diesel) via features (`backend/Cargo.toml`)
  * Configuration via environment variables and/or `backend/repository.toml` (see `docs/REPOSITORY_PATTERN.md`)
* Existing Docker Compose:

  * `docker/docker-compose.yml` starts Postgres and the app (Streamlit) with `DATABASE_URL` to enable Postgres in Rust.

### 2.4 Contracts and Data Schemas

* JSON schema for schedules exists at `backend/docs/schedule.schema.json`
* The application already operates on a set of “queries” that are strong candidates for HTTP endpoints:

  * `store_schedule`, `list_schedules`, `get_sky_map_data`, `get_distribution_data`, `get_visibility_map_data`, `get_schedule_timeline_data`, `get_insights_data`, `get_trends_data`, `get_compare_data`, `get_validation_report`, etc. (registered under `backend/src/routes/*`)

---

## 3) Target State (TO-BE): Decoupled Client–Server Architecture

### 3.1 Logical Diagram

```
┌──────────────────────────┐
│ Web Frontend (React+TS) │
│ - UI                   │
│ - Interactive plots    │
│ - REST + SSE consumer  │
└─────────────▲──────────┘
              │ HTTP/JSON + SSE
              ▼
┌──────────────────────────┐
│ Rust Backend (axum)     │
│ - REST API              │
│ - SSE progress / jobs   │
│ - Reuse services/db     │
└─────────────▲──────────┘
              │ Repository traits
              ▼
┌──────────────────────────┐
│ Persistence              │
│ - Postgres (prod)        │
│ - Local/in-memory (dev)  │
└──────────────────────────┘
```

### 3.2 Key Migration Principle

The “real backend” **already exists** as a Rust library (`backend/`) with:

* models and parsing,
* business services,
* abstracted repository layer,
* mostly serializable DTOs.

What is missing:

* a **transport layer** (HTTP),
* **explicit contracts** for a web client (OpenAPI/JSON),
* a **decoupled frontend**.

---

## 4) Concrete Reuse of the Existing Code (What Is Preserved)

### 4.1 Rust: Reusable Core Without Rewrites

* `backend/src/models/*`: parsing and base domain types
* `backend/src/db/*`: repository pattern + Postgres/Local (already implemented)
* `backend/src/services/*`: logic for all dashboard views (sky map, timeline, insights, etc.)
* `backend/src/routes/*`: current catalog of “use cases” (excellent basis for mapping to HTTP endpoints)

---

## 5) Gap Analysis (What’s Missing to Reach TO-BE)

### 5.1 Transport / API

* Today: in-process invocation (Python → Rust extension)
* Target: network invocation (HTTP), with:

  * stable JSON serialization,
  * request validation,
  * API versioning,
  * CORS and payload limits

### 5.2 Concurrency and Multi-Client Support

* Today: Streamlit manages “sessions” per user, backend runs in the same process
* Target: multiple concurrent clients against a single service

  * Open decision: **dataset visibility** across clients (shared vs namespaced)
  * Minimal recommendation: introduce an anonymous `client_id` for logical namespacing if required

### 5.3 Operations (Production)

* Today: Docker Compose tailored for Streamlit + Postgres
* Target: deployment of Rust service + static frontend + Postgres, with observability (logs/metrics) and health checks

---

## 6) Proposed API Design (Direct Mapping From Existing Functions)

### 6.1 Suggested REST Endpoints (v1)

Based on functions currently exposed under `backend/src/routes/*`:

* `GET /health`

  * Reusable from `backend/src/db/services.rs::health_check`
* `POST /v1/schedules`

  * Body: `{ name, schedule_json, visibility_json? }`
  * Equivalent to `routes::landing::store_schedule`
* `GET /v1/schedules`

  * Equivalent to `routes::landing::list_schedules`
* `GET /v1/schedules/{schedule_id}/sky-map`

  * Equivalent to `routes::skymap::get_sky_map_data`
* `GET /v1/schedules/{schedule_id}/distributions`
* `GET /v1/schedules/{schedule_id}/visibility-map`
* `GET /v1/schedules/{schedule_id}/timeline`
* `GET /v1/schedules/{schedule_id}/insights`
* `GET /v1/schedules/{schedule_id}/trends?bins=&bandwidth=&points=`
* `GET /v1/schedules/{schedule_id}/validation-report`
* `GET /v1/schedules/{schedule_id}/compare/{other_id}`

### 6.2 SSE (Progress / Jobs)

Primary candidate: “upload + analytics computation” (currently `store_schedule` may trigger analytics population).

* `POST /v1/schedules` may return:

  * immediate response (201 + `schedule_id`) with async job, or
  * 202 + `job_id`
* SSE:

  * `GET /v1/jobs/{job_id}/events` (Server-Sent Events) for progress: parsing, persistence, analytics phase 1/2/3

---

## 7) Proposed Technical Structure of the Rust Backend (Minimal and Evolvable)

### 7.1 Recommended Option (Minimizes Changes)

* Keep `backend/` as a library (`tsi-rust`)
* Add a new HTTP service bin/crate (e.g. `backend/http_server/` or `backend/server/`) that:

  * depends on `tsi-rust` (reuses `models/db/services`),
  * exposes axum handlers (REST + SSE),
  * initializes the repository explicitly at startup (instead of lazy init)

Advantage: maximum reuse, localized changes, existing tests remain valid.

### 7.2 Important Consideration

Some PyO3 routes use ad-hoc `tokio::runtime::Runtime` instances (e.g. `routes/landing.rs::list_schedules`).
In an axum server, an async runtime already exists; therefore handlers should call the async functions in `db/services` directly, without creating per-request runtimes. Python bindings can be removed after full migration.

---

## 8) Web Frontend (React + TypeScript) — Migration Scope

### 8.1 What Streamlit Currently Does and Must Be Migrated

* Current pages (`src/tsi/pages/*`) and components (`src/tsi/components/*`) translate into React views.
* Visualizations:

  * Plotly (Python) → Plotly.js / ECharts (client-side)
  * Graphs (if applicable) → Cytoscape.js or Sigma.js
* State:

  * `st.session_state` → frontend state management (React state / Zustand / Redux, depending on scale)

### 8.2 Data Contracts

Most backend Rust DTOs already derive `Serialize/Deserialize` in route modules, facilitating direct JSON contracts (with review for:

* numeric types (`f64` vs int),
* wrappers (`ScheduleId`) and their serialization,
* stable compatibility for the web client).

---

## 9) Incremental Migration Plan (Aligned With the Current Repo)

### Phase 0 — Technical Inventory (Fast)

* Catalog target endpoints directly from `backend/src/routes/*` and their usage in `src/tsi/services/backend_service.py`
* Identify which DTOs are already serializable and which need adjustments

### Phase 1 — Minimal HTTP Backend (axum)

* Implement:

  * `GET /health`
  * `GET /v1/schedules`
  * `POST /v1/schedules`
* Repository by environment:

  * `LocalRepository` by default (dev)
  * `PostgresRepository` in prod (already supported via env/compose)

### Phase 2 — Visualization Endpoints (Streamlit Parity)

* Add endpoints for: sky map, distributions, visibility map, timeline, insights, trends, validation, compare
* Validate payload sizes and latencies (large datasets)

### Phase 3 — Initial Web Client

* Minimal UI:

  * upload/list schedules
  * sky map + one metrics panel to validate end-to-end

### Phase 4 — SSE / Jobs (Operational Robustness)

* Progress streaming for uploads and expensive computations
* Timeouts, cancellation (if applicable), and concurrency limits

### Phase 5 — Progressive Streamlit Deprecation

* Keep Streamlit as an optional internal tool (or remove it) once the web frontend reaches parity

---

## 10) Technical Risks and Mitigations

* **Contract changes (Python DTOs vs JSON)**: version the API (`/v1`) and freeze contracts with OpenAPI + contract tests
* **Concurrency and client isolation**: define policy (shared vs per-`client_id`); implement logical namespacing if privacy is required
* **Large payloads**: server-side pagination/filtering, compression, and SSE/streaming where applicable
* **Postgres execution**: Diesel + env (`DATABASE_URL`) already supported; reinforce migrations and startup health checks

---

## 11) Validation (Existing Tests and Extensions)

* Rust already has a strong test base in `backend/tests/*` (including Local and Postgres repository tests)
* Add:

  * HTTP API tests (contract tests) for axum endpoints
  * End-to-end integration tests (frontend ↔ backend) in CI once the frontend exists

---

## 12) Notes (For Architecture / Stakeholders)

1. Schedules are global, not associated to any anonymous `client_id`?
2. The Python wheel (`maturin`) shall be fully replaced by the Rust service.
3. Long-running computation model: asynchronous jobs with SSE + `job_id`
4. API versioning and documentation strategy (OpenAPI/Swagger, JSON Schema, etc.)

---

## 13) Repository References (Anchor Points)

* Rust backend (core): `backend/src/models/`, `backend/src/db/`, `backend/src/services/`, `backend/src/routes/`
* Bindings/packaging: `pyproject.toml`, `backend/src/lib.rs`, `backend/src/api.rs`
* Current Streamlit app: `src/tsi/app.py`, `src/tsi/pages/`, `src/tsi/services/backend_service.py`
* Persistence and design docs: `docs/REPOSITORY_PATTERN.md`, `docs/POSTGRES_ETL_DB_DESIGN.md`, `docker/docker-compose.yml`
* Schedule schema: `backend/docs/schedule.schema.json`

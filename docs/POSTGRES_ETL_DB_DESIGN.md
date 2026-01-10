# Postgres + Diesel: ETL & Database Design for Schedule Uploads

This document proposes a revised ETL process and a new **Postgres** schema (managed via **Diesel**) to persist all data produced when a schedule is uploaded, while matching the repository pattern used by the Rust backend.

## Requirements (from request)

- Use Postgres and Diesel.
- Follow the repository traits/pattern already present in `backend/src/db/repository/`.
- **All period collections** (e.g., dark periods, visibility periods) are stored as **JSON arrays in a single column**.
- **Do not store histogram products** (e.g., visibility map / histogram bins) in the database.

## Non-goals

- Not redesigning the Python UI layer; it continues to call the Rust API.
- Not persisting derived histogram/bin caches; those are computed on demand.

---

## 1) What “data produced on upload” means in this repo

Uploading a schedule produces four durable “families” of data that the dashboard and services consume:

1. **Raw schedule entities**
   - Schedule metadata (name, checksum, upload time)
   - Scheduling blocks (priority, target coordinates, constraints, durations)
   - Period collections: `dark_periods`, per-block `visibility_periods`, per-block `scheduled_period`
2. **Block-level analytics** (fast path for dashboard queries)
   - Denormalized block metrics: requested hours, total visibility hours, counts, scheduled flag, priority bucket, etc.
3. **Schedule-level summary analytics**
   - Aggregates used by landing/metrics panels (counts, means, scheduling rate, etc.)
4. **Validation results**
   - Per-block validation issues (impossible blocks, warnings, errors)

This design stores (1)–(4). Histogram-like products are recomputed at query time.

---

## 2) Revised ETL Process (Postgres-first)

### Phase A — Extract

1. Receive `schedule_json` (+ optional `possible_periods_json`; dark periods may come from upload or local config).
2. Parse into domain models (`Schedule`, `SchedulingBlock`, `Period`) using existing Serde parsing (`backend/src/models/schedule.rs`).
3. Compute schedule checksum (SHA-256) for idempotency.

### Phase B — Transform

1. Validate blocks (domain constraints: RA/Dec ranges, durations, scheduled period consistency).
2. Derive per-block metrics (no database reads required):
   - `requested_hours = requested_duration_sec / 3600`
   - `total_visibility_hours` + `num_visibility_periods` from `visibility_periods`
   - `scheduled = scheduled_period present`
   - `elevation_range_deg = max_alt - min_alt`
   - `priority_bucket` (configured in Rust; stored as an integer)
3. Derive schedule-level products:
   - `possible_periods` as a merged/unioned list (optional but recommended to support `ScheduleRepository::fetch_possible_periods` without scanning all blocks)
   - `summary_analytics` aggregates

### Phase C — Load (single transaction)

1. Insert (or upsert) the schedule row keyed by `checksum`.
2. Insert schedule blocks for that schedule (bulk insert).
3. Insert block-level analytics for that schedule (bulk insert).
4. Insert schedule summary analytics row.
5. Insert validation results rows.

All load steps should run in a single DB transaction so a failed upload cannot leave partial state.

**Histogram note:** visibility-map-like datasets are computed on demand from `schedule_blocks` and/or `schedule_block_analytics`, never written.

---

## 3) Schema Design (Diesel-ready)

### 3.1 Naming and types

- Primary keys: `BIGSERIAL` (Diesel `BigInt`).
- Times:
  - `uploaded_at`: `TIMESTAMPTZ` (real wall clock time).
  - MJD values remain `DOUBLE PRECISION` inside JSON period objects; analytics fields can keep MJD as `DOUBLE PRECISION` for filtering/sorting.
- Period JSON columns: `JSONB` arrays of `{ "start": <f64>, "stop": <f64> }`.

### 3.2 Tables

#### `schedules`

Stores schedule identity + schedule-level period arrays.

```sql
CREATE TABLE schedules (
  schedule_id           BIGSERIAL PRIMARY KEY,
  schedule_name         TEXT NOT NULL,
  checksum              TEXT NOT NULL UNIQUE,
  uploaded_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

  -- Period arrays (requirement: JSON arrays in a single column)
  dark_periods_json     JSONB NOT NULL DEFAULT '[]'::jsonb,
  possible_periods_json JSONB NOT NULL DEFAULT '[]'::jsonb,

  -- Optional: preserve uploaded payload for audit/replay
  raw_schedule_json     JSONB
);

CREATE INDEX schedules_uploaded_at_idx ON schedules (uploaded_at DESC);
```

#### `schedule_blocks`

Stores all atomic scheduling blocks belonging to a schedule. Period collections are JSON arrays.

```sql
CREATE TABLE schedule_blocks (
  scheduling_block_id        BIGSERIAL PRIMARY KEY,
  schedule_id                BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,

  -- ID as provided by the input schedule (kept for compare/matching)
  source_block_id            BIGINT NOT NULL,
  original_block_id          TEXT,

  -- Core block fields (aligned to Rust DTOs)
  priority                   DOUBLE PRECISION NOT NULL,
  requested_duration_sec     INTEGER NOT NULL,
  min_observation_sec        INTEGER NOT NULL,
  target_ra_deg              DOUBLE PRECISION NOT NULL,
  target_dec_deg             DOUBLE PRECISION NOT NULL,

  -- Constraints (kept columnar for query speed)
  min_altitude_deg           DOUBLE PRECISION,
  max_altitude_deg           DOUBLE PRECISION,
  min_azimuth_deg            DOUBLE PRECISION,
  max_azimuth_deg            DOUBLE PRECISION,
  constraint_start_mjd       DOUBLE PRECISION,
  constraint_stop_mjd        DOUBLE PRECISION,

  -- Period arrays (requirement)
  visibility_periods_json    JSONB NOT NULL DEFAULT '[]'::jsonb,

  -- Store scheduled period as an array of length 0 or 1 to respect the “array” requirement
  scheduled_periods_json     JSONB NOT NULL DEFAULT '[]'::jsonb,

  created_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT schedule_blocks_unique_per_schedule
    UNIQUE (schedule_id, source_block_id),

  CONSTRAINT schedule_blocks_valid_durations
    CHECK (requested_duration_sec >= 0
       AND min_observation_sec >= 0
       AND min_observation_sec <= requested_duration_sec),

  CONSTRAINT visibility_periods_is_array
    CHECK (jsonb_typeof(visibility_periods_json) = 'array'),

  CONSTRAINT scheduled_periods_is_array
    CHECK (jsonb_typeof(scheduled_periods_json) = 'array')
);

CREATE INDEX schedule_blocks_schedule_id_idx ON schedule_blocks (schedule_id);
CREATE INDEX schedule_blocks_source_id_idx ON schedule_blocks (source_block_id);
```

#### `schedule_block_analytics`

Precomputed per-block metrics used by the dashboard “fast path”.

```sql
CREATE TABLE schedule_block_analytics (
  schedule_id              BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,
  scheduling_block_id      BIGINT NOT NULL REFERENCES schedule_blocks(scheduling_block_id) ON DELETE CASCADE,

  -- Denormalized metrics
  priority_bucket          SMALLINT NOT NULL,
  requested_hours          DOUBLE PRECISION NOT NULL,
  total_visibility_hours   DOUBLE PRECISION NOT NULL,
  num_visibility_periods   INTEGER NOT NULL,
  elevation_range_deg      DOUBLE PRECISION,

  -- Scheduling result
  scheduled                BOOLEAN NOT NULL,
  scheduled_start_mjd      DOUBLE PRECISION,
  scheduled_stop_mjd       DOUBLE PRECISION,

  -- Validation flags (optional but useful for filtering)
  validation_impossible    BOOLEAN NOT NULL DEFAULT FALSE,

  created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),

  PRIMARY KEY (schedule_id, scheduling_block_id)
);

CREATE INDEX schedule_block_analytics_schedule_id_idx ON schedule_block_analytics (schedule_id);
CREATE INDEX schedule_block_analytics_scheduled_idx ON schedule_block_analytics (schedule_id, scheduled);
CREATE INDEX schedule_block_analytics_priority_idx ON schedule_block_analytics (schedule_id, priority_bucket);
```

#### `schedule_summary_analytics`

One row per schedule with summary metrics.

```sql
CREATE TABLE schedule_summary_analytics (
  schedule_id                 BIGINT PRIMARY KEY REFERENCES schedules(schedule_id) ON DELETE CASCADE,

  total_blocks                INTEGER NOT NULL,
  scheduled_blocks            INTEGER NOT NULL,
  unscheduled_blocks          INTEGER NOT NULL,
  impossible_blocks           INTEGER NOT NULL,

  scheduling_rate             DOUBLE PRECISION NOT NULL,

  priority_mean               DOUBLE PRECISION,
  priority_median             DOUBLE PRECISION,
  priority_scheduled_mean     DOUBLE PRECISION,
  priority_unscheduled_mean   DOUBLE PRECISION,

  visibility_total_hours      DOUBLE PRECISION NOT NULL,
  requested_mean_hours        DOUBLE PRECISION,

  created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### `schedule_validation_results`

Stores the detailed validation output generated during upload/analytics.

```sql
CREATE TABLE schedule_validation_results (
  validation_id          BIGSERIAL PRIMARY KEY,
  schedule_id            BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,
  scheduling_block_id    BIGINT NOT NULL REFERENCES schedule_blocks(scheduling_block_id) ON DELETE CASCADE,

  status                 TEXT NOT NULL,  -- valid | warning | error | impossible
  issue_type             TEXT,
  issue_category         TEXT,
  criticality            TEXT,
  field_name             TEXT,
  current_value          TEXT,
  expected_value         TEXT,
  description            TEXT,

  created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX schedule_validation_results_schedule_id_idx ON schedule_validation_results (schedule_id);
CREATE INDEX schedule_validation_results_status_idx ON schedule_validation_results (schedule_id, status);
```

### 3.3 What we intentionally do NOT store

- Any “bin” or histogram table, including visibility-map/histogram caches.
- Any pre-rendered plots (those are UI artifacts).

---

## 4) Mapping to Repository Traits

This schema is designed to implement the current trait surface in `backend/src/db/repository/`.

### `ScheduleRepository`

- `store_schedule(schedule)`
  - Insert/upsert `schedules` by `checksum`
  - Insert `schedule_blocks` (with JSON arrays for periods)
- `get_schedule(schedule_id)`
  - Load from `schedules` + `schedule_blocks`
  - Deserialize `*_periods_json` (`jsonb`) into `Vec<Period>` / `Option<Period>` (via 0/1-length array)
- `list_schedules()`
  - Select from `schedules`
- `fetch_dark_periods(schedule_id)`
  - Read `schedules.dark_periods_json`
- `fetch_possible_periods(schedule_id)`
  - Read `schedules.possible_periods_json` (recommended) or compute union from blocks if left empty

### `AnalyticsRepository`

- `populate_schedule_analytics(schedule_id)`
  - Insert into `schedule_block_analytics` + `schedule_summary_analytics`
  - Never writes histogram bins
- `fetch_analytics_blocks_for_*`
  - Reads from `schedule_block_analytics` (and joins to `schedule_blocks` when raw fields like RA/Dec or original IDs are needed)

### `ValidationRepository`

- `insert_validation_results(results)`
  - Bulk insert into `schedule_validation_results`
  - Optionally update `schedule_block_analytics.validation_impossible`

### `VisualizationRepository`

- `fetch_visibility_map_data(...)` / `fetch_blocks_for_histogram(...)`
  - Compute from `schedule_blocks`/`schedule_block_analytics` at query time
  - No persistence needed

---

## 5) Diesel Implementation Notes

- Use Diesel migrations to create tables and indexes:
  - `diesel migration generate init_postgres_schema`
  - Put the DDL above into `up.sql` (and `DROP TABLE ...` into `down.sql`).
- Map `JSONB` columns to `serde_json::Value` in Rust DB models and convert to/from `Vec<Period>` using Serde.
- Prefer batched inserts (`insert_into(...).values(&vec)`) for `schedule_blocks` and analytics rows.
- Use a Postgres pool (`deadpool-diesel` or `r2d2`) and keep the repository interface async by running Diesel operations in a blocking threadpool (Tokio `spawn_blocking`) or by using an async-friendly wrapper.

---

## 6) Query Patterns (why this works for the dashboard)

- **Sky map**: `schedule_blocks` (RA/Dec, priority, scheduled period) + `schedule_block_analytics.priority_bucket`.
- **Distributions/Trends/Insights**: read mostly from `schedule_block_analytics` (fast, no JSON parsing).
- **Timeline**: scheduled blocks from `schedule_block_analytics` (start/stop MJD) + `schedules.dark_periods_json`.
- **Compare**: join analytics from two schedules on `schedule_blocks.source_block_id` (or `original_block_id` if stable).

---

## 7) Open Decisions / Follow-ups

- **Union strategy for `possible_periods_json`**: store as-is from input, or store the merged union for schedule-level overlays.
- **Strict JSON validation**: add deeper checks (optional) to ensure each array element contains numeric `start`/`stop`.
- **Partitioning**: for very large datasets, consider partitioning `schedule_blocks` and `schedule_block_analytics` by `schedule_id`.


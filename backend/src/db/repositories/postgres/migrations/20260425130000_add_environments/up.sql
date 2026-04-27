-- The environments table and schedules.environment_id already exist from prior
-- migrations (20260424120000, 20260424130000) but were created with a minimal
-- schema (INTEGER PK, name, created_at only).  This migration upgrades them to
-- the full fingerprint schema without dropping existing data.

-- ── 1. Promote INTEGER PKs to BIGINT (required by Diesel Int8 schema) ─────────
ALTER TABLE schedules DROP CONSTRAINT schedules_environment_id_fkey;

ALTER TABLE environments ALTER COLUMN environment_id TYPE BIGINT;
ALTER SEQUENCE environments_environment_id_seq AS BIGINT MAXVALUE 9223372036854775807;

ALTER TABLE schedules ALTER COLUMN environment_id TYPE BIGINT;

ALTER TABLE schedules
  ADD CONSTRAINT schedules_environment_id_fkey
    FOREIGN KEY (environment_id) REFERENCES environments(environment_id)
    ON DELETE SET NULL;

-- ── 2. Add fingerprint columns ────────────────────────────────────────────────
-- NULL values mean the environment has not been initialised yet.
ALTER TABLE environments
  ADD COLUMN period_start_mjd  DOUBLE PRECISION,
  ADD COLUMN period_end_mjd    DOUBLE PRECISION,
  ADD COLUMN lat_deg           DOUBLE PRECISION,
  ADD COLUMN lon_deg           DOUBLE PRECISION,
  ADD COLUMN elevation_m       DOUBLE PRECISION,
  ADD COLUMN blocks_hash       TEXT;

-- ── 3. Constraints and indexes ────────────────────────────────────────────────
ALTER TABLE environments
  ADD CONSTRAINT environments_name_unique UNIQUE (name);

ALTER TABLE environments
  ADD CONSTRAINT environments_structure_consistent CHECK (
    (period_start_mjd IS NULL
     AND period_end_mjd IS NULL
     AND lat_deg IS NULL
     AND lon_deg IS NULL
     AND elevation_m IS NULL
     AND blocks_hash IS NULL)
    OR
    (period_start_mjd IS NOT NULL
     AND period_end_mjd IS NOT NULL
     AND lat_deg IS NOT NULL
     AND lon_deg IS NOT NULL
     AND elevation_m IS NOT NULL
     AND blocks_hash IS NOT NULL)
  );

CREATE INDEX environments_blocks_hash_idx ON environments (blocks_hash);

-- ── 4. Per-environment preschedule cache ──────────────────────────────────────
CREATE TABLE environment_preschedule (
  environment_id  BIGINT PRIMARY KEY
                    REFERENCES environments(environment_id)
                    ON DELETE CASCADE,
  payload_json    JSONB NOT NULL,
  computed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── 5. Index the FK on schedules ──────────────────────────────────────────────
CREATE INDEX schedules_environment_id_idx ON schedules (environment_id);

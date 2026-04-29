-- Support both:
-- 1. older databases where environments + schedules.environment_id already
--    exist with a minimal INTEGER-based schema, and
-- 2. fresh databases bootstrapped from the current checked-in migration set,
--    where neither exists yet.

CREATE TABLE IF NOT EXISTS environments (
  environment_id BIGSERIAL PRIMARY KEY,
  name           TEXT NOT NULL,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE schedules
  ADD COLUMN IF NOT EXISTS environment_id BIGINT;

-- ── 1. Promote INTEGER PKs to BIGINT (required by Diesel Int8 schema) ─────────
ALTER TABLE schedules DROP CONSTRAINT IF EXISTS schedules_environment_id_fkey;

ALTER TABLE environments ALTER COLUMN environment_id TYPE BIGINT;
DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_class
    WHERE relkind = 'S'
      AND relname = 'environments_environment_id_seq'
  ) THEN
    EXECUTE
      'ALTER SEQUENCE environments_environment_id_seq AS BIGINT MAXVALUE 9223372036854775807';
  END IF;
END
$$;

ALTER TABLE schedules ALTER COLUMN environment_id TYPE BIGINT;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'schedules_environment_id_fkey'
  ) THEN
    ALTER TABLE schedules
      ADD CONSTRAINT schedules_environment_id_fkey
        FOREIGN KEY (environment_id) REFERENCES environments(environment_id)
        ON DELETE SET NULL;
  END IF;
END
$$;

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
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'environments_name_unique'
  ) THEN
    ALTER TABLE environments
      ADD CONSTRAINT environments_name_unique UNIQUE (name);
  END IF;
END
$$;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'environments_structure_consistent'
  ) THEN
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
  END IF;
END
$$;

CREATE INDEX IF NOT EXISTS environments_blocks_hash_idx ON environments (blocks_hash);

-- ── 4. Per-environment preschedule cache ──────────────────────────────────────
CREATE TABLE IF NOT EXISTS environment_preschedule (
  environment_id  BIGINT PRIMARY KEY
                    REFERENCES environments(environment_id)
                    ON DELETE CASCADE,
  payload_json    JSONB NOT NULL,
  computed_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── 5. Index the FK on schedules ──────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS schedules_environment_id_idx ON schedules (environment_id);

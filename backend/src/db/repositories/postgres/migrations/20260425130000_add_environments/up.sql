-- Environments group schedules that share the same scheduling structure
-- (period × scheduling-blocks × observer-location). The structure is
-- inferred from the first schedule assigned to the environment;
-- subsequent schedules are validated against this fingerprint.
--
-- A per-environment "preschedule" cache stores the visibility / dark-period
-- payload that is otherwise computed once per schedule, so importing
-- additional matching schedules is essentially free.

CREATE TABLE environments (
  environment_id     BIGSERIAL PRIMARY KEY,
  name               TEXT NOT NULL UNIQUE,

  -- Structure fingerprint. NULL columns mean the environment has not
  -- been initialised yet (no member schedule has been assigned).
  period_start_mjd   DOUBLE PRECISION,
  period_end_mjd     DOUBLE PRECISION,
  lat_deg            DOUBLE PRECISION,
  lon_deg            DOUBLE PRECISION,
  elevation_m        DOUBLE PRECISION,
  blocks_hash        TEXT,

  created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT environments_structure_consistent CHECK (
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
  )
);

CREATE INDEX environments_blocks_hash_idx ON environments (blocks_hash);

-- Per-environment shared preschedule payload (visibility, dark periods,
-- astronomical nights). One row per initialised environment.
CREATE TABLE environment_preschedule (
  environment_id     BIGINT PRIMARY KEY
                       REFERENCES environments(environment_id)
                       ON DELETE CASCADE,
  payload_json       JSONB NOT NULL,
  computed_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Optional FK from schedules to environments. Deleting an environment
-- unassigns its members; deleting a schedule does not affect the env.
ALTER TABLE schedules
  ADD COLUMN environment_id BIGINT
    REFERENCES environments(environment_id)
    ON DELETE SET NULL;

CREATE INDEX schedules_environment_id_idx ON schedules (environment_id);

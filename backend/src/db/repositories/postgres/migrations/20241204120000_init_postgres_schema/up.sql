-- Postgres schema for schedule ingestion and analytics

CREATE TABLE schedules (
  schedule_id           BIGSERIAL PRIMARY KEY,
  schedule_name         TEXT NOT NULL,
  checksum              TEXT NOT NULL UNIQUE,
  uploaded_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

  dark_periods_json     JSONB NOT NULL DEFAULT '[]'::jsonb,
  possible_periods_json JSONB NOT NULL DEFAULT '[]'::jsonb,
  raw_schedule_json     JSONB
);

CREATE INDEX schedules_uploaded_at_idx ON schedules (uploaded_at DESC);

CREATE TABLE schedule_blocks (
  scheduling_block_id        BIGSERIAL PRIMARY KEY,
  schedule_id                BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,

  source_block_id            BIGINT NOT NULL,
  original_block_id          TEXT,

  priority                   DOUBLE PRECISION NOT NULL,
  requested_duration_sec     INTEGER NOT NULL,
  min_observation_sec        INTEGER NOT NULL,
  target_ra_deg              DOUBLE PRECISION NOT NULL,
  target_dec_deg             DOUBLE PRECISION NOT NULL,

  min_altitude_deg           DOUBLE PRECISION,
  max_altitude_deg           DOUBLE PRECISION,
  min_azimuth_deg            DOUBLE PRECISION,
  max_azimuth_deg            DOUBLE PRECISION,
  constraint_start_mjd       DOUBLE PRECISION,
  constraint_stop_mjd        DOUBLE PRECISION,

  visibility_periods_json    JSONB NOT NULL DEFAULT '[]'::jsonb,
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

CREATE TABLE schedule_block_analytics (
  schedule_id              BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,
  scheduling_block_id      BIGINT NOT NULL REFERENCES schedule_blocks(scheduling_block_id) ON DELETE CASCADE,

  priority_bucket          SMALLINT NOT NULL,
  requested_hours          DOUBLE PRECISION NOT NULL,
  total_visibility_hours   DOUBLE PRECISION NOT NULL,
  num_visibility_periods   INTEGER NOT NULL,
  elevation_range_deg      DOUBLE PRECISION,

  scheduled                BOOLEAN NOT NULL,
  scheduled_start_mjd      DOUBLE PRECISION,
  scheduled_stop_mjd       DOUBLE PRECISION,

  validation_impossible    BOOLEAN NOT NULL DEFAULT FALSE,

  created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),

  PRIMARY KEY (schedule_id, scheduling_block_id)
);

CREATE INDEX schedule_block_analytics_schedule_id_idx ON schedule_block_analytics (schedule_id);
CREATE INDEX schedule_block_analytics_scheduled_idx ON schedule_block_analytics (schedule_id, scheduled);
CREATE INDEX schedule_block_analytics_priority_idx ON schedule_block_analytics (schedule_id, priority_bucket);

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

CREATE TABLE schedule_validation_results (
  validation_id          BIGSERIAL PRIMARY KEY,
  schedule_id            BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,
  scheduling_block_id    BIGINT NOT NULL REFERENCES schedule_blocks(scheduling_block_id) ON DELETE CASCADE,

  status                 TEXT NOT NULL,
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

-- Per-schedule EST algorithm trace.
--
-- Stores the structured trace emitted by the EST scheduler when a schedule
-- is produced.  Each row carries the parsed `summary` event (algorithm
-- configuration + run-level aggregates) and the array of per-iteration events.
--
-- A schedule may have at most one trace.  Deleting the schedule cascades
-- to its trace.
CREATE TABLE est_traces (
    schedule_id BIGINT      PRIMARY KEY
                            REFERENCES schedules(schedule_id)
                            ON DELETE CASCADE,
    summary     JSONB       NOT NULL,
    iterations  JSONB       NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

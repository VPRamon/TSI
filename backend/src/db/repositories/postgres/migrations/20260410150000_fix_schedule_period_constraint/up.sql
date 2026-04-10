-- Fix schedule_period_json validation to match Period serialization
-- used by the Rust API (`start_mjd` / `end_mjd`).

ALTER TABLE schedules DROP CONSTRAINT IF EXISTS schedule_period_is_valid;

-- Normalize any legacy rows that may have been written with start/stop keys.
UPDATE schedules
SET schedule_period_json = jsonb_build_object(
  'start_mjd', schedule_period_json->'start',
  'end_mjd', schedule_period_json->'stop'
)
WHERE jsonb_typeof(schedule_period_json) = 'object'
  AND schedule_period_json ? 'start'
  AND schedule_period_json ? 'stop'
  AND NOT (schedule_period_json ? 'start_mjd')
  AND NOT (schedule_period_json ? 'end_mjd');

ALTER TABLE schedules
ADD CONSTRAINT schedule_period_is_valid
CHECK (
  jsonb_typeof(schedule_period_json) = 'object'
  AND schedule_period_json ? 'start_mjd'
  AND schedule_period_json ? 'end_mjd'
  AND jsonb_typeof(schedule_period_json->'start_mjd') = 'number'
  AND jsonb_typeof(schedule_period_json->'end_mjd') = 'number'
);

COMMENT ON COLUMN schedules.schedule_period_json IS 'Overall time window for the schedule in MJD format. Required for visibility map rendering. Example: {"start_mjd": 62115.0, "end_mjd": 62121.0}';

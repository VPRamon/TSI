ALTER TABLE schedules DROP CONSTRAINT IF EXISTS schedule_period_is_valid;

ALTER TABLE schedules
ADD CONSTRAINT schedule_period_is_valid
CHECK (
  jsonb_typeof(schedule_period_json) = 'object'
  AND schedule_period_json ? 'start'
  AND schedule_period_json ? 'stop'
  AND jsonb_typeof(schedule_period_json->'start') = 'number'
  AND jsonb_typeof(schedule_period_json->'stop') = 'number'
);

COMMENT ON COLUMN schedules.schedule_period_json IS 'Overall time window for the schedule in MJD format. Required for visibility map rendering. Example: {"start": 62115.0, "stop": 62121.0}';

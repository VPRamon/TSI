-- Add schedule_period_json column to store the overall schedule time window

ALTER TABLE schedules 
ADD COLUMN schedule_period_json JSONB NOT NULL;

-- Add constraint to ensure it's a valid Period object
ALTER TABLE schedules
ADD CONSTRAINT schedule_period_is_valid
CHECK (
  jsonb_typeof(schedule_period_json) = 'object'
  AND schedule_period_json ? 'start'
  AND schedule_period_json ? 'stop'
  AND jsonb_typeof(schedule_period_json->'start') = 'number'
  AND jsonb_typeof(schedule_period_json->'stop') = 'number'
);

-- Add comment for documentation
COMMENT ON COLUMN schedules.schedule_period_json IS 'Overall time window for the schedule in MJD format. Required for visibility map rendering. Example: {"start": 62115.0, "stop": 62121.0}';

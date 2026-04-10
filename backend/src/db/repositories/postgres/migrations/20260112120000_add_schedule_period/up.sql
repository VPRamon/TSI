-- Add schedule_period_json column to store the overall schedule time window

ALTER TABLE schedules 
ADD COLUMN schedule_period_json JSONB NOT NULL;

-- Add constraint to ensure it's a valid Period object
ALTER TABLE schedules
ADD CONSTRAINT schedule_period_is_valid
CHECK (
  jsonb_typeof(schedule_period_json) = 'object'
  AND schedule_period_json ? 'start_mjd'
  AND schedule_period_json ? 'end_mjd'
  AND jsonb_typeof(schedule_period_json->'start_mjd') = 'number'
  AND jsonb_typeof(schedule_period_json->'end_mjd') = 'number'
);

-- Add comment for documentation
COMMENT ON COLUMN schedules.schedule_period_json IS 'Overall time window for the schedule in MJD format. Required for visibility map rendering. Example: {"start_mjd": 62115.0, "end_mjd": 62121.0}';

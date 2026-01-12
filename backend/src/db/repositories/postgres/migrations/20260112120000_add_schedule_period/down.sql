-- Revert schedule_period_json column addition

ALTER TABLE schedules DROP CONSTRAINT IF EXISTS schedule_period_is_valid;
ALTER TABLE schedules DROP COLUMN IF EXISTS schedule_period_json;

-- Remove observer_location_json and astronomical_night_periods_json from schedules table

ALTER TABLE schedules DROP CONSTRAINT IF EXISTS astronomical_night_periods_is_array;
ALTER TABLE schedules DROP CONSTRAINT IF EXISTS observer_location_is_valid;
ALTER TABLE schedules DROP COLUMN IF EXISTS astronomical_night_periods_json;
ALTER TABLE schedules DROP COLUMN IF EXISTS observer_location_json;

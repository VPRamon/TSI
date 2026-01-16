-- Add observer_location_json and astronomical_night_periods_json to schedules table

-- Add observer location (lat, lon, elevation) - REQUIRED
ALTER TABLE schedules 
ADD COLUMN observer_location_json JSONB NOT NULL;

-- Add astronomical night periods (computed from observer location + schedule period)
ALTER TABLE schedules
ADD COLUMN astronomical_night_periods_json JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Add constraint to ensure observer_location is a valid GeographicLocation object
ALTER TABLE schedules
ADD CONSTRAINT observer_location_is_valid
CHECK (
  jsonb_typeof(observer_location_json) = 'object'
  AND observer_location_json ? 'latitude'
  AND observer_location_json ? 'longitude'
  AND jsonb_typeof(observer_location_json->'latitude') = 'number'
  AND jsonb_typeof(observer_location_json->'longitude') = 'number'
  AND (observer_location_json->'latitude')::float >= -90.0
  AND (observer_location_json->'latitude')::float <= 90.0
  AND (observer_location_json->'longitude')::float >= -180.0
  AND (observer_location_json->'longitude')::float <= 180.0
);

-- Add constraint to ensure astronomical_night_periods is an array
ALTER TABLE schedules
ADD CONSTRAINT astronomical_night_periods_is_array
CHECK (
  jsonb_typeof(astronomical_night_periods_json) = 'array'
);

-- Add comments for documentation
COMMENT ON COLUMN schedules.observer_location_json IS 'Geographic location of the observatory (latitude, longitude, optional elevation_m). REQUIRED. Example: {"latitude": 28.7624, "longitude": -17.8892, "elevation_m": 2396}';
COMMENT ON COLUMN schedules.astronomical_night_periods_json IS 'Computed astronomical night periods (Sun altitude < -18°) based on observer location and schedule period. Array of Period objects in MJD. Example: [{"start": 62115.5, "stop": 62115.8}]';

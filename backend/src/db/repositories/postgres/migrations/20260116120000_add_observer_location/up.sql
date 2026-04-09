-- Add observer_location_json and astronomical_night_periods_json to schedules table

-- Add observer location (lat, lon, elevation) - REQUIRED
ALTER TABLE schedules 
ADD COLUMN observer_location_json JSONB NOT NULL;

-- Add astronomical night periods (computed from observer location + schedule period)
ALTER TABLE schedules
ADD COLUMN astronomical_night_periods_json JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Add constraint to ensure observer_location is a valid Geodetic position object
ALTER TABLE schedules
ADD CONSTRAINT observer_location_is_valid
CHECK (
  jsonb_typeof(observer_location_json) = 'object'
  AND observer_location_json ? 'lat_deg'
  AND observer_location_json ? 'lon_deg'
  AND observer_location_json ? 'height'
  AND jsonb_typeof(observer_location_json->'lat_deg') = 'number'
  AND jsonb_typeof(observer_location_json->'lon_deg') = 'number'
  AND jsonb_typeof(observer_location_json->'height') = 'number'
  AND (observer_location_json->'lat_deg')::float >= -90.0
  AND (observer_location_json->'lat_deg')::float <= 90.0
  AND (observer_location_json->'lon_deg')::float >= -180.0
  AND (observer_location_json->'lon_deg')::float <= 180.0
);

-- Add constraint to ensure astronomical_night_periods is an array
ALTER TABLE schedules
ADD CONSTRAINT astronomical_night_periods_is_array
CHECK (
  jsonb_typeof(astronomical_night_periods_json) = 'array'
);

-- Add comments for documentation
COMMENT ON COLUMN schedules.observer_location_json IS 'Geodetic position of the observatory (lon_deg, lat_deg, height). REQUIRED. Example: {"lon_deg": -17.8892, "lat_deg": 28.7624, "height": 2396}';
COMMENT ON COLUMN schedules.astronomical_night_periods_json IS 'Computed astronomical night periods (Sun altitude < -18°) based on observer location and schedule period. Array of Period objects in MJD. Example: [{"start": 62115.5, "stop": 62115.8}]';

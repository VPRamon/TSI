DROP INDEX IF EXISTS schedules_environment_id_idx;
DROP TABLE IF EXISTS environment_preschedule;
DROP INDEX IF EXISTS environments_blocks_hash_idx;
ALTER TABLE environments DROP CONSTRAINT IF EXISTS environments_structure_consistent;
ALTER TABLE environments DROP CONSTRAINT IF EXISTS environments_name_unique;
ALTER TABLE environments
  DROP COLUMN IF EXISTS period_start_mjd,
  DROP COLUMN IF EXISTS period_end_mjd,
  DROP COLUMN IF EXISTS lat_deg,
  DROP COLUMN IF EXISTS lon_deg,
  DROP COLUMN IF EXISTS elevation_m,
  DROP COLUMN IF EXISTS blocks_hash;
-- Note: environments table and schedules.environment_id remain (created by prior migrations)

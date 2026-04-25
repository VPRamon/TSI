DROP INDEX IF EXISTS schedules_environment_id_idx;
ALTER TABLE schedules DROP COLUMN IF EXISTS environment_id;
DROP TABLE IF EXISTS environment_preschedule;
DROP TABLE IF EXISTS environments;

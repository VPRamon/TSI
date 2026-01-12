-- Remove gap metrics columns from schedule_summary_analytics table
ALTER TABLE schedule_summary_analytics
DROP COLUMN gap_count,
DROP COLUMN gap_mean_hours,
DROP COLUMN gap_median_hours;

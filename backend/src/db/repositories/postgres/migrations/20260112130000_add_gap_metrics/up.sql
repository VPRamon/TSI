-- Add gap metrics columns to schedule_summary_analytics table
ALTER TABLE schedule_summary_analytics
ADD COLUMN gap_count INTEGER,
ADD COLUMN gap_mean_hours DOUBLE PRECISION,
ADD COLUMN gap_median_hours DOUBLE PRECISION;

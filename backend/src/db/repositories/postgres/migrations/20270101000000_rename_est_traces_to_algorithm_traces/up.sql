-- Rename `est_traces` → `algorithm_traces` and add an `algorithm`
-- discriminator column.  TSI no longer privileges any specific scheduling
-- algorithm; per-schedule traces are now algorithm-agnostic blobs.
--
-- Existing rows are EST-derived, so the new column is back-filled with
-- `'est'`.  Future inserts must specify the algorithm explicitly.
ALTER TABLE est_traces RENAME TO algorithm_traces;
ALTER TABLE algorithm_traces ADD COLUMN algorithm TEXT NOT NULL DEFAULT 'est';
ALTER TABLE algorithm_traces ALTER COLUMN algorithm DROP DEFAULT;

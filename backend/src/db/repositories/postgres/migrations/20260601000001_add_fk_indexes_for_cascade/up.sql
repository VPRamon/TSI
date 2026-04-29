-- Index FK columns that point at schedule_blocks(scheduling_block_id) so
-- ON DELETE CASCADE from schedule_blocks does not seq-scan the dependent
-- tables once per deleted row.
--
-- Without these indexes, deleting a schedule with N blocks forces Postgres
-- to perform N sequential scans of `schedule_block_analytics` and
-- `schedule_validation_results`, which is the dominant cost of
-- `DELETE FROM schedules WHERE schedule_id = $1`.

CREATE INDEX IF NOT EXISTS schedule_block_analytics_block_id_idx
    ON schedule_block_analytics (scheduling_block_id);

CREATE INDEX IF NOT EXISTS schedule_validation_results_block_id_idx
    ON schedule_validation_results (scheduling_block_id);

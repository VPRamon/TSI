-- Add block_name column to schedule_blocks to store a human-readable name
-- (e.g. the target name) alongside the machine-readable original_block_id.

ALTER TABLE schedule_blocks
    ADD COLUMN IF NOT EXISTS block_name TEXT NOT NULL DEFAULT '';

COMMENT ON COLUMN schedule_blocks.block_name IS
    'Human-readable name for the scheduling block (e.g. target name). '
    'Set by the import adapter; empty string when not provided.';

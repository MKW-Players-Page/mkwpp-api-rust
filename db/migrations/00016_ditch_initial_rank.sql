ALTER TABLE scores ADD COLUMN was_wr BOOLEAN DEFAULT FALSE NOT NULL;
UPDATE scores SET was_wr = TRUE WHERE initial_rank = 1;
ALTER TABLE scores DROP COLUMN initial_rank;
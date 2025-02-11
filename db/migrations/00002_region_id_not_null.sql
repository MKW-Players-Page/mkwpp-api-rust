UPDATE players SET region_id = 1 WHERE region_id IS NULL;
ALTER TABLE players ALTER COLUMN region_id SET NOT NULL;
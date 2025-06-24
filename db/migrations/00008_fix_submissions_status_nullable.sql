UPDATE submissions SET status = 'pending';
UPDATE edit_submissions SET status = 'pending';
ALTER TABLE edit_submissions ALTER COLUMN status SET DEFAULT 'pending';
ALTER TABLE submissions ALTER COLUMN status SET DEFAULT 'pending';
ALTER TABLE edit_submissions ALTER COLUMN status SET NOT NULL;
ALTER TABLE submissions ALTER COLUMN status SET NOT NULL;

ALTER TABLE edit_submissions ALTER COLUMN reviewer_id DROP NOT NULL;
ALTER TABLE edit_submissions ALTER COLUMN reviewed_at DROP NOT NULL;
ALTER TABLE submissions ALTER COLUMN reviewer_id DROP NOT NULL;
ALTER TABLE submissions ALTER COLUMN reviewed_at DROP NOT NULL;
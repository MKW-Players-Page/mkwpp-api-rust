ALTER TABLE ONLY submissions ALTER COLUMN submitted_at SET DEFAULT NOW();
ALTER TABLE edit_submission RENAME TO edit_submissions;
ALTER TABLE ONLY edit_submissions ALTER COLUMN submitted_at SET DEFAULT NOW();
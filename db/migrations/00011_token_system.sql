ALTER TABLE activation_tokens RENAME TO tokens;
ALTER TABLE tokens DROP COLUMN activated;
CREATE TYPE token_type AS ENUM ('activation', 'password_reset');
ALTER TABLE tokens ADD COLUMN token_type token_type;
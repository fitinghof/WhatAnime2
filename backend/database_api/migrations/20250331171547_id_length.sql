-- Add migration script here
ALTER TABLE reports
  ALTER COLUMN user_id TYPE VARCHAR(32),
  ALTER COLUMN handled_by TYPE VARCHAR(32);
ALTER TABLE users
 ALTER COLUMN id TYPE VARCHAR(32);

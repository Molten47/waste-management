ALTER TABLE users ADD COLUMN password_hash TEXT;

-- emails are case-insensitive: Ade@x.com and ade@x.com must be one account
CREATE UNIQUE INDEX users_email_lower_idx ON users (LOWER(email));
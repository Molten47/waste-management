ALTER TABLE users
    ADD COLUMN otp_hash       TEXT,
    ADD COLUMN otp_expires_at TIMESTAMPTZ,
    ADD COLUMN otp_attempts   SMALLINT NOT NULL DEFAULT 0;
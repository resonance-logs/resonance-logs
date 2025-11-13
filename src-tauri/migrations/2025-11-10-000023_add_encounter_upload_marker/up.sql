-- Track when encounters have been uploaded so auto-upload can skip finished rows
ALTER TABLE encounters ADD COLUMN uploaded_at_ms BIGINT;

-- Index speeds up pending encounter scans (NULL filter)
CREATE INDEX IF NOT EXISTS idx_encounters_uploaded_at ON encounters(uploaded_at_ms);

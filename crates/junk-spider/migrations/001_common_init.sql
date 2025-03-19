-- Build the `intervals` table, inserting the common values.
CREATE TABLE IF NOT EXISTS common.intervals (
    interval_pk SERIAL,
    interval VARCHAR
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_interval ON common.intervals(interval);

INSERT INTO common.intervals (interval)
SELECT unnest(ARRAY['30m', '1h', '1d', '1w']::text[])
WHERE NOT EXISTS (
    SELECT 1 FROM common.intervals WHERE interval IN ('30m', '1h', '1d', '1w')
);
-- Build the `intervals` table, inserting the common values.
CREATE TABLE IF NOT EXISTS common.ref_time_intervals (
    interval_pk SERIAL PRIMARY KEY,
    interval VARCHAR
);

INSERT INTO common.ref_time_intervals (interval)
SELECT unnest(ARRAY['1m', '15m', '30m', '1h', '1d', '1w', '1mo']::text[])
WHERE NOT EXISTS (
    SELECT 1 FROM common.ref_time_intervals WHERE interval IN ('1m', '15m', '30m', '1h', '1d', '1w', '1mo')
);


CREATE TABLE IF NOT EXISTS common.ref_time_interval_aliases (
    alias VARCHAR(10),
    interval_fk SERIAL REFERENCES common.ref_time_intervals(interval_pk)
);
ALTER TABLE common.ref_time_interval_aliases
ADD CONSTRAINT common_time_interval_aliases UNIQUE (alias, interval_fk);

-- INSERT INTO common.ref_time_intervals (interval_pk, interval)
-- VALUES (
--     (1, '1m'),
--     (2, '15m'),
--     (3, '30m'),
--     (4, '1h'),
--     (5, '1d'),
--     (6, '1w'),
--     (7, '1mo')
-- ) ON CONFLICT (interval_pk, interval) DO NOTHING;

INSERT INTO common.ref_time_interval_aliases (alias, interval_fk)
VALUES 
    ('1m', 1),
    ('1min', 1),
    ('1 min', 1),
    ('1minute', 1),
    ('1 minute', 1),

    ('15m', 2),
    ('15min', 2),
    ('15 min', 2),
    ('15minutes', 2),
    ('15 minutes', 2),

    ('30m', 3),
    ('30min', 3),
    ('30 min', 3),
    ('30minutes', 3),
    ('30 minutes', 3),

    ('1h', 4),
    ('1hr', 4),
    ('1hour', 4),
    ('1 hour', 4),

    ('1d', 5),
    ('1day', 5),
    ('1 day', 5),

    ('1w', 6),
    ('1wk', 6),
    ('1week', 6),
    ('1 week', 6),

    ('1mo', 7),
    ('1mnth', 7),
    ('1month', 7),
    ('1 month', 7)
ON CONFLICT (alias, interval_fk) DO NOTHING;

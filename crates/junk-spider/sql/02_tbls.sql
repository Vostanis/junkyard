--------------------------------------------------------------------------------------
-- COMMON
--------------------------------------------------------------------------------------- 

-- data intervals, e.g., 1m, 5m, 1hr, 1d, 1wk, 1mo, 1yr
-- currently only supporting 30m, 1h, 1d, 1w; only 1d being used
CREATE TABLE IF NOT EXISTS common.intervals (
	pk SMALLSERIAL PRIMARY KEY,
	interval CHAR(3) NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_interval ON common.intervals(interval);

INSERT INTO common.intervals (interval)
SELECT unnest(ARRAY['30m', '1h', '1d', '1w']::text[])
WHERE NOT EXISTS (
    SELECT 1 FROM common.intervals WHERE interval IN ('30m', '1h', '1d', '1w')
);

--------------------------------------------------------------------------------------
-- CRYPTO
--------------------------------------------------------------------------------------

-- crypto pairs
CREATE TABLE IF NOT EXISTS crypto.symbols (
	pk SERIAL PRIMARY KEY,
	symbol VARCHAR NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_symbol ON crypto.symbols(symbol);

-- price data, including number of trades
CREATE TABLE IF NOT EXISTS crypto.prices (
	symbol_pk INT,
	time TIMESTAMP WITH TIME ZONE NOT NULL,
	interval_pk SMALLINT,
	opening FLOAT,
	high FLOAT,
	low FLOAT,
	closing FLOAT,
	volume FLOAT,
	trades BIGINT,
	source_pk SMALLINT,
	PRIMARY KEY (symbol_pk, time, interval_pk, source_pk)
);

-- which broker the data came from, e.g. binance, kucoin, mexc
CREATE TABLE IF NOT EXISTS crypto.sources (
	pk SMALLSERIAL PRIMARY KEY,
	source VARCHAR NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_source ON crypto.sources(source);

--------------------------------------------------------------------------------------
-- STOCK
--------------------------------------------------------------------------------------

-- CREATE TABLE IF NOT EXISTS stock.prices;
-- CREATE TABLE IF NOT EXISTS stock.metrics;

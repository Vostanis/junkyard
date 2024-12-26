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

CREATE TABLE IF NOT EXISTS stock.tickers (
	pk SERIAL PRIMARY KEY,
	symbol VARCHAR NOT NULL,
	industry VARCHAR,
	nation CHAR(4) NOT NULL,
	INDEX idx_symbol (symbol),
	INDEX idx_nation (nation)
);

CREATE TABLE IF NOT EXISTS stock.prices (
	symbol_pk INT,
	interval_pk SMALLINT,
	dated DATE NOT NULL,
	opening FLOAT,
	high FLOAT,
	low FLOAT,
	closing FLOAT,
	volume BIGINT,
	PRIMARY KEY (symbol_pk, interval_pk, dated),
	INDEX idx_symbol_pk (symbol_pk),
	INDEX idx_interval_pk (interval_pk),
	INDEX idx_dated (dated)
)
PARTITION BY HASH(symbol_pk);

CREATE TABLE IF NOT EXISTS stock.metrics (
	symbol_pk INT NOT NULL,
	metric_pk INT NOT NULL,
	acc_pk INT NOT NULL,
	dated DATE NOT NULL,
	val FLOAT NOT NULL,
	PRIMARY KEY (symbol_pk, metric_pk, dated, val),
	INDEX idx_symbol_pk (symbol_pk),
	INDEX idx_metric_pk (metric_pk),
	INDEX idx_dated (dated)
)
PARTITION BY HASH(symbol_pk);

-- Metrics Library (e.g., pk: 1 -> name: "Revenues")
CREATE TABLE IF NOT EXISTS stock.metrics_lib (
	pk SERIAL PRIMARY KEY,
	metric VARCHAR NOT NULL,
	INDEX idx_metric (metric)
);

-- Accounting Standards (e.g., pk: 1 -> "US-GAAP")
CREATE TABLE IF NOT EXISTS stock.acc_stds (
	pk SERIAL PRIMARY KEY,
	symbol VARCHAR,
	INDEX idx_symbol (symbol)
);

-- CREATE TABLE IF NOT EXISTS stock.filings;

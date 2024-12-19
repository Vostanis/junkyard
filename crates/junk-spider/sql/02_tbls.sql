
--------------------------------------------------------------------------------------
-- COMMON
--------------------------------------------------------------------------------------- 

-- data intervals, e.g., 1m, 5m, 1hr, 1d, 1wk, 1mo, 1yr
CREATE TABLE IF NOT EXISTS common.intervals (
	pk SERIAL PRIMARY KEY,
	interval CHAR(3) NOT NULL
);

--------------------------------------------------------------------------------------
-- CRYPTO
--------------------------------------------------------------------------------------

-- crypto pairs
CREATE TABLE IF NOT EXISTS crypto.symbols (
	pk SERIAL PRIMARY KEY,
	symbol VARCHAR NOT NULL
);

-- price data, including number of trades
CREATE TABLE IF NOT EXISTS crypto.prices (
	symbol_pk INT,
	time TIMESTAMP,
	interval_pk INT,
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
	pk SERIAL PRIMARY KEY,
	source VARCHAR NOT NULL
);

--------------------------------------------------------------------------------------
-- STOCK
--------------------------------------------------------------------------------------

-- CREATE TABLE IF NOT EXISTS stock.prices;
-- CREATE TABLE IF NOT EXISTS stock.metrics;

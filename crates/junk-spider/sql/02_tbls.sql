
--------------------------------------------------------------------------------------
-- COMMON
--------------------------------------------------------------------------------------- 

-- data intervals, e.g., 1m, 5m, 1hr, 1d, 1wk, 1mo, 1yr
CREATE TABLE IF NOT EXISTS common.intervals (
	id SERIAL PRIMARY KEY,
	interval CHAR(3) NOT NULL
);

--------------------------------------------------------------------------------------
-- CRYPTO
--------------------------------------------------------------------------------------

-- crypto pairs
CREATE TABLE IF NOT EXISTS crypto.symbols (
	id SERIAL PRIMARY KEY,
	symbol VARCHAR(10) NOT NULL,
);

-- price data, including number of trades
CREATE TABLE IF NOT EXISTS crypto.prices (
	id INT,
	time TIMESTAMP,
	interval INT,
	opening FLOAT,
	high FLOAT,
	low FLOAT,
	closing FLOAT,
	volume FLOAT,
	trades BIGINT,
	source SMALLINT,
);

-- which broker the data came from, e.g. binance, kucoin, mexc
CREATE TABLE IF NOT EXISTS crypto.sources (
	id SERIAL PRIMARY KEY,
	broker CHAR(8) NOT NULL
);

--------------------------------------------------------------------------------------
-- STOCK
--------------------------------------------------------------------------------------

-- CREATE TABLE IF NOT EXISTS stock.prices;
-- CREATE TABLE IF NOT EXISTS stock.metrics;

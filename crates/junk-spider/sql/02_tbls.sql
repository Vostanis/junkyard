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
	dt TIMESTAMP WITH TIME ZONE NOT NULL,
	interval_pk SMALLINT,
	opening FLOAT,
	high FLOAT,
	low FLOAT,
	closing FLOAT,
	volume FLOAT,
	trades BIGINT,
	source_pk SMALLINT,
	PRIMARY KEY (symbol_pk, dt, interval_pk, source_pk)
);
CREATE INDEX IF NOT EXISTS idx_symbol_pk ON crypto.prices(symbol_pk);
CREATE INDEX IF NOT EXISTS idx_interval_pk ON crypto.prices(interval_pk);
CREATE INDEX IF NOT EXISTS idx_dt ON crypto.prices(dt);

-- which broker the data came from, e.g. binance, kucoin, mexc
CREATE TABLE IF NOT EXISTS crypto.sources (
	pk SMALLSERIAL PRIMARY KEY,
	source VARCHAR NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_source ON crypto.sources(source);

--------------------------------------------------------------------------------------
-- STOCK
--------------------------------------------------------------------------------------

-- ticker/title list
CREATE TABLE IF NOT EXISTS stock.tickers (
	pk SERIAL PRIMARY KEY,
	cik CHAR(10),
	ticker VARCHAR NOT NULL,
	title VARCHAR NOT NULL,
	industry VARCHAR,
	nation CHAR(4) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ticker ON stock.tickers(ticker);
CREATE INDEX IF NOT EXISTS idx_nation ON stock.tickers(nation);

-- ticker list
CREATE TABLE IF NOT EXISTS stock.symbols (
	pk SERIAL,
	symbol VARCHAR NOT NULL,
	title VARCHAR NOT NULL,
	industry VARCHAR,
	file_code CHAR(10),
	nation CHAR(4) NOT NULL,
	PRIMARY KEY (symbol, title, nation, industry)
);
CREATE INDEX IF NOT EXISTS idx_symbol ON stock.symbols(symbol);
CREATE INDEX IF NOT EXISTS idx_title ON stock.symbols(title);

-- prices value table
CREATE TABLE IF NOT EXISTS stock.prices (
	symbol_pk INT,
	interval_pk SMALLINT,
	dt TIMESTAMP WITH TIME ZONE NOT NULL,
	opening FLOAT,
	high FLOAT,
	low FLOAT,
	closing FLOAT,
	adj_close FLOAT,
	volume BIGINT,
	PRIMARY KEY (symbol_pk, interval_pk, dt)
);

-- metrics value table
CREATE TABLE IF NOT EXISTS stock.metrics (
	symbol_pk INT NOT NULL,
	metric_pk INT NOT NULL,
	acc_pk INT NOT NULL,
	dated DATE NOT NULL,
	year SMALLINT,
	period CHAR(2),
	form VARCHAR,
	val FLOAT NOT NULL,
	accn VARCHAR,
	PRIMARY KEY (symbol_pk, metric_pk, acc_pk, dated, val, accn)
);
CREATE INDEX IF NOT EXISTS idx_symbol_pk ON stock.metrics(symbol_pk);
CREATE INDEX IF NOT EXISTS idx_dated ON stock.metrics(dated);

-- metrics library (e.g., pk: 1 -> name: "Revenues")
CREATE TABLE IF NOT EXISTS stock.metrics_lib (
	pk SERIAL PRIMARY KEY,
	metric VARCHAR NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_metric ON stock.metrics_lib(metric);

-- accounting standards (e.g., pk: 1 -> "US-GAAP")
CREATE TABLE IF NOT EXISTS stock.acc_stds (
	pk SERIAL PRIMARY KEY,
	accounting VARCHAR NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_accounting ON stock.acc_stds(accounting);

-- CREATE TABLE IF NOT EXISTS stock.filings;

--------------------------------------------------------------------------------------
-- ECONOMIC
--------------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS econ.fred (
	dated DATE,
	metric VARCHAR,
	val FLOAT,
	PRIMARY KEY (dated, metric, val)
);

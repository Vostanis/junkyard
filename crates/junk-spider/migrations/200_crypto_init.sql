-- Reference list of cryptocurrency pairs.
CREATE TABLE IF NOT EXISTS crypto.ref_symbols (
	symbol_pk SERIAL PRIMARY KEY,
	symbol VARCHAR NOT NULL
);

-- Cryptocurrency price data, including number of trades and sources.
CREATE TABLE IF NOT EXISTS crypto.fact_prices (
	symbol_pk INT,
	dt TIMESTAMP WITH TIME ZONE NOT NULL,
	interval_pk SMALLINT,
	open FLOAT,
	high FLOAT,
	low FLOAT,
	close FLOAT,
	volume FLOAT,
	trades BIGINT,
	source_pk SMALLINT,
	PRIMARY KEY (symbol_pk, dt, interval_pk, source_pk)
);
CREATE INDEX IF NOT EXISTS idx_symbol_pk ON crypto.fact_prices(symbol_pk);
CREATE INDEX IF NOT EXISTS idx_interval_pk ON crypto.fact_prices(interval_pk);
CREATE INDEX IF NOT EXISTS idx_dt ON crypto.fact_prices(dt);

-- Reference list of brokers sourcing the data, 
-- e.g., binance, kucoin, mexc
CREATE TABLE IF NOT EXISTS crypto.ref_sources (
	source_pk SMALLSERIAL PRIMARY KEY,
	source VARCHAR NOT NULL
);
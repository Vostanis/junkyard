-- Build core components of `stock` schema.

-- Stock asset information table.
CREATE TABLE IF NOT EXISTS stock.dim_companies (
	symbol_pk SERIAL,
	symbol VARCHAR,
	name VARCHAR,
	industry VARCHAR,
	nation CHAR(2),
	description VARCHAR,
	PRIMARY KEY (symbol, title, nation, industry)
);
CREATE INDEX IF NOT EXISTS idx_symbol ON stock.dim_companies(symbol);
CREATE INDEX IF NOT EXISTS idx_title ON stock.dim_companies(name);

-- Stage raw price data, prior to adding metrics.
CREATE TABLE IF NOT EXISTS stock.stg_prices (
	symbol_pk INT,
	interval_pk INT,
	dt TIMESTAMP WITH TIME ZONE,
	open FLOAT,
	high FLOAT,
	low FLOAT,
	close FLOAT,
	adj_close FLOAT,
	volume FLOAT,
	PRIMARY KEY (symbol_pk, interval_pk, dt)
);

-- Metrics only requires 1 raw data table.
-- Inferred metrics are subsequently inserted.
CREATE TABLE IF NOT EXISTS stock.fact_metrics (
	symbol_pk INT NOT NULL,
	metric_pk INT NOT NULL,
	taxonomy_pk INT NOT NULL,
	start_date DATE,
	end_date DATE NOT NULL,
	filing_date DATE NOT NULL,
	year SMALLINT,
	period CHAR(2) NOT NULL,
	form VARCHAR NOT NULL,
	val FLOAT NOT NULL,
	accn VARCHAR,
	frame VARCHAR
);
ALTER TABLE stock.metrics
ADD CONSTRAINT stock_metric_entries UNIQUE (symbol_pk, metric_pk, taxonomy_pk, start_date, end_date, filing_date, period, form, val, accn, frame);
CREATE INDEX IF NOT EXISTS idx_symbol_pk ON stock.metrics(symbol_pk);
CREATE INDEX IF NOT EXISTS idx_end_date ON stock.metrics(end_date);
CREATE INDEX IF NOT EXISTS idx_filing_date ON stock.metrics(filing_date);

-- View of only the Quarterly metrics; common shortcut for inferring missing metrics.
-- CREATE VIEW IF NOT EXISTS stock.fact_metrics_quarterly;

-- Reference list of metric names.
CREATE TABLE IF NOT EXISTS stock.ref_metrics (
    metric_pk SERIAL PRIMARY KEY,
    metric VARCHAR
);

-- Reference list of taxonomies.
-- us-gaap, ifrs-full, dei, srt
CREATE TABLE IF NOT EXISTS stock.ref_taxonomy (
    taxonomy_pk SERIAL PRIMARY KEY,
    taxonomy VARCHAR
);
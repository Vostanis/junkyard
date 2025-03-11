CREATE TABLE IF NOT EXISTS stock.stg_sec_documents (
	filing_date DATE,
	form_type VARCHAR,
	url VARCHAR,
	xml XML
);
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.fact_insider_transactions;

CREATE TABLE IF NOT EXISTS stock.dim_companies (
	symbol_pk SERIAL,
	symbol VARCHAR,
	name VARCHAR,
	industry VARCHAR,
	nation CHAR(2),
	description VARCHAR,
	PRIMARY KEY (symbol, title, nation, industry)
);

CREATE TABLE IF NOT EXISTS stock.stg_prices (
	symbol_pk INT,
	interval_pk INT,
	dt TIMESTAMP WITH TIME ZONE,
	open FLOAT,
	high FLOAT,
	low FLOAT,
	close FLOAT,
	adj_close FLOAT,
	volume FLOAT
);

CREATE MATERIALIZED VIEW IF NOT EXISTS stock.fact_prices (
WITH	moving_avgs AS (

	),
	percentage_change AS (

	)
	
SELECT	symbol_pk,
	interval_pk,
	dt,
	open,
	high,
	low,
	close,
	adj_close,
	adj_close_20ma,
	adj_close_50ma,
	adj_close_200ma,
	volume

FROM	stock.stg_prices
LEFT JOIN moving_avgs
LEFT JOIN percentage_change
);

CREATE TABLE IF NOT EXISTS stock.fact_metrics;
CREATE VIEW IF NOT EXISTS stock.fact_metrics_quarterly;
CREATE TABLE IF NOT EXISTS stock.ref_metrics;
CREATE TABLE IF NOT EXISTS stock.ref_taxonomy; -- us-gaap, ifrs-full, dei, srt

CREATE MATERIALIZED VIEW IF NOT EXISTS stock.std_financials;

CREATE MATERIALIZED VIEW IF NOT EXISTS api.stocks; -- JSON collection of std_financials & prices

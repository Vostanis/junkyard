-- Clean the "title" & "industry" of special characters.
CREATE MATERIALIZED VIEW IF NOT EXISTS api.stock_symbols
SELECT
    symbol,
    REGEXP_REPLACE(title, '[''\\\/]', '', 'g') AS title, -- clean any special characters
    REGEXP_REPLACE(industry, '[''\\\/]', '', 'g') AS industry,
    nation
FROM stock.symbols;
CREATE INDEX IF NOT EXISTS idx_mv_symbol ON api.stock_symbols(symbol);

-- Client-facing price table.
CREATE MATERIALIZED VIEW IF NOT EXISTS api.stock_prices AS 
SELECT
	comp.symbol,
	pr.dt,
	pr.open,
	pr.high,
	pr.low,
	pr.close,
	pr.adj_close,
	pr.volume
FROM 
	stock.fact_prices pr
INNER JOIN 
	stock.dim_companies comp USING(symbol_pk);
CREATE INDEX IF NOT EXISTS idx_mv_symbol ON api.stock_prices(symbol);

-- Client-facing metrics table.
-- CREATE MATERIALIZED VIEW IF NOT EXISTS api.stock_metrics AS ;

CREATE MATERIALIZED VIEW IF NOT EXISTS api.ref_stock_datasets AS
SELECT
	*
FROM 
	stock.dim_companies comp
INNER JOIN 
	stock.fact_prices pr USING(symbol_pk)
INNER JOIN 
	stock.std_financials fin USING(symbol_pk);
CREATE INDEX IF NOT EXISTS idx_mv_symbol ON api.ref_stock_datasets(symbol);

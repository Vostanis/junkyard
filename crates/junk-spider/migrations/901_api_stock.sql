CREATE MATERIALIZED VIEW api.stock_symbols (
SELECT
    symbol,
    REGEXP_REPLACE(title, '[''\\\/]', '', 'g') AS title, -- clean any special characters
    REGEXP_REPLACE(industry, '[''\\\/]', '', 'g') AS industry,
    nation
FROM stock.symbols
);

CREATE VIEW api.stock_prices;
CREATE VIEW api.stock_metrics;
CREATE MATERIALIZED VIEW api.stock_aggregates;

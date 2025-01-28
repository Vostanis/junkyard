-- stock metrics 
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.metrics_matv AS
SELECT
	sy.symbol,
	sy.title,
	m.start_date,
	m.end_date,
	m.year,
	m.period,
	m.form,
	m.frame,
	mlib.metric,
	m.val,
	acc.accounting
FROM stock.symbols AS sy
INNER JOIN stock.metrics AS m
	ON sy.pk = m.symbol_pk
INNER JOIN stock.metrics_lib AS mlib
	ON mlib.pk = m.metric_pk
INNER JOIN stock.acc_stds AS acc
	ON acc.pk = m.acc_pk
;

-- stock prices
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.prices_matv AS

-- preprocess 2nd gen metrics
WITH 
-- moving averages for volume & price
moving_average_cte AS (
SELECT
        pr.symbol_pk,
        pr.interval_pk,
        pr.dt,
        pr.volume,
	AVG(pr.volume) OVER (
		PARTITION BY pr.symbol_pk, pr.interval_pk
		ORDER BY pr.dt
		ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
	) as volume_7ma,
	AVG(pr.volume) OVER (
		PARTITION BY pr.symbol_pk, pr.interval_pk
		ORDER BY pr.dt
		ROWS BETWEEN 90 PRECEDING AND CURRENT ROW
	) as volume_90ma,
	AVG(pr.volume) OVER (
		PARTITION BY pr.symbol_pk, pr.interval_pk
		ORDER BY pr.dt
		ROWS BETWEEN 365 PRECEDING AND CURRENT ROW
	) as volume_365ma
FROM stock.prices AS pr
)

-- price change percentages; per interval
price_change_cte AS (
SELECT
	pr.symbol_pk,
	pr.interval_pk,
	pr.dt,
	pr.adj_close,
        (pr.adj_close - LAG(pr.adj_close) OVER (
            PARTITION BY pr.symbol_pk, pr.interval_pk
            ORDER BY pr.datetime
        )) / LAG(pr.adj_close) OVER (
            PARTITION BY pr.symbol_pk, pr.interval_pk
            ORDER BY pr.datetime
        ) * 100 AS adj_close_pct_change
FROM stock.prices as pr
)

SELECT
	sy.symbol,
	sy.title,
	pr.dt,
	intv.interval,
	pr.opening,
	pr.high,
	pr.low,
	pr.closing,
	pr.adj_close,
	pr.volume,
	ma.volume_7ma,
	ma.volume_90ma,
	ma.volume_365ma
	
FROM stock.symbols AS sy
INNER JOIN stock.prices AS pr
	ON sy.pk = pr.symbol_pk
INNER JOIN common.intervals AS intv
	ON intv.pk = pr.interval_pk

-- moving averages
LEFT JOIN moving_average_cte AS ma
	ON pr.symbol_pk = ma.symbol_pk
	AND pr.interval_pk = ma.interval_pk
	AND pr.dt = ma.dt

-- percentage changes
;

-- crypto prices
CREATE MATERIALIZED VIEW IF NOT EXISTS crypto.prices_matv AS
SELECT
	sy.symbol,
	so.source,
	iv.interval,
	pr.opening,
	pr.high,
	pr.low,
	pr.closing,
	pr.volume,
	pr.trades
FROM crypto.symbols AS sy
INNER JOIN crypto.prices AS pr
	ON sy.pk = pr.symbol_pk
INNER JOIN crypto.sources AS so
	ON so.pk = pr.source_pk
INNER JOIN common.intervals AS iv
	ON iv.pk = pr.interval_pk
;

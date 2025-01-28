--------------------------------------------------------------------------
--
-- RELATION TO THE DATABASE
-- ========================
--
-- MATERIALIZED VIEWs are used as the entrypoint for publicised data;
-- 1. they are precomputed, and so are fast to query;
-- 2. a layer of transformation between the raw input and the output is
--    cleaner for debugging;
-- 3. and they can be refreshed on a schedule, or on-demand.
--
-- Therefore, when users query data (via an app or API), they're querying
-- from one of the following views.
--
--------------------------------------------------------------------------

-- =====================================================================
-- STOCKS
-- =====================================================================

-- Stock Metrics
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

-- Stock Prices
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.prices_matv AS

WITH 
-- moving averages for volume & price (adj. close), per 7, 90, and 365 x interval
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
		) AS volume_7ma,
		AVG(pr.volume) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk
			ORDER BY pr.dt
			ROWS BETWEEN 89 PRECEDING AND CURRENT ROW
		) AS volume_90ma,
		AVG(pr.volume) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk
			ORDER BY pr.dt
			ROWS BETWEEN 364 PRECEDING AND CURRENT ROW
		) AS volume_365ma
	FROM stock.prices AS pr
),

-- price (adj. close) change percentages, per interval
percentage_change_cte AS (
	SELECT
		pr.symbol_pk,
		pr.interval_pk,
		pr.dt,
		pr.adj_close,
		CASE
			-- error case: division by zero
			WHEN LAG(pr.adj_close) OVER (
			    PARTITION BY pr.symbol_pk, pr.interval_pk
			    ORDER BY pr.dt
			) = 0 THEN NULL
			ELSE (pr.adj_close - LAG(pr.adj_close) OVER (
			    PARTITION BY pr.symbol_pk, pr.interval_pk
			    ORDER BY pr.dt
			)) / LAG(pr.adj_close) OVER (
			    PARTITION BY pr.symbol_pk, pr.interval_pk
			    ORDER BY pr.dt
			) * 100
		END AS perc
	FROM stock.prices AS pr
)

SELECT
	sy.symbol,
	sy.title,
	pr.dt,
	intv.interval,
	pc.perc,
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
LEFT JOIN percentage_change_cte AS pc
	ON pr.symbol_pk = pc.symbol_pk
	AND pr.interval_pk = pc.interval_pk
	AND pr.dt = pc.dt
;

-- =====================================================================
-- CRYPTO
-- =====================================================================

-- Crypto Prices
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

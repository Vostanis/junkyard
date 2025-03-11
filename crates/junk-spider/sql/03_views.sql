--------------------------------------------------------------------------
--
-- RELATION TO THE DATABASE
-- ========================
--
-- MATERIALIZED VIEWs are used as the entrypoint for publicised data, 
-- since:
-- 		1. they are precomputed, and so are fast to query;
-- 		2. a layer of transformation between the raw input and the output 
--		   is cleaner for debugging;
-- 		3. and they can be refreshed on a schedule, or on-demand.
--
-- Therefore, when users query data (via an app or API) they're querying
-- from one of the VIEWs built with this script.
--
--------------------------------------------------------------------------

-- =====================================================================
-- STOCKS
-- =====================================================================

-- Stock Metrics
-- DROP MATERIALIZED VIEW stock.metrics_matv;
-- CREATE MATERIALIZED VIEW stock.metrics_matv AS (
-- SELECT
-- 	s.symbol,
-- 	s.title,
-- 	l.metric,
-- 	m.start_date,
-- 	m.end_date,
-- 	m.val,
-- 	ARRAY_AGG(DISTINCT m.form) AS forms,
-- 	ARRAY_AGG(DISTINCT m.accn) AS accns
	
-- FROM stock.metrics m
-- INNER JOIN stock.symbols s ON m.symbol_pk = s.pk
-- INNER JOIN stock.metrics_lib l ON m.metric_pk = l.pk

-- WHERE 
-- 	((m.end_date - m.start_date) <= 120) OR (start_date IS NULL AND m.frame LIKE '%I')
-- GROUP BY 
-- 	s.symbol,
-- 	s.title,
-- 	l.metric,
-- 	m.start_date,
-- 	m.end_date,
-- 	m.val
-- ORDER BY 
-- 	m.start_date DESC
-- );

DROP VIEW IF EXISTS stock.metrics_q;
CREATE VIEW stock.metrics_q AS (
SELECT *
FROM stock.metrics m
WHERE 
		((m.end_date - m.start_date) <= 100) -- typical quarterly entries
	OR (m.start_date IS NULL AND m.frame LIKE '%I') -- instantaneous data
	OR (m.period = 'I') -- inferred
);

-- Stock Prices
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.prices_matv AS
WITH 
-- moving averages for volume & price (adj. close), per 7, 90, and 365 x interval
moving_average_cte AS (
	SELECT
		pr.symbol_pk,
		pr.interval_pk,
		pr.dt,
		AVG(pr.adj_close) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk
			ORDER BY pr.dt
			ROWS BETWEEN 19 PRECEDING AND CURRENT ROW
		) AS adj_close_20ma,
		AVG(pr.adj_close) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk
			ORDER BY pr.dt
			ROWS BETWEEN 49 PRECEDING AND CURRENT ROW
		) AS adj_close_50ma,
		AVG(pr.adj_close) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk
			ORDER BY pr.dt
			ROWS BETWEEN 199 PRECEDING AND CURRENT ROW
		) AS adj_close_200ma,
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
	ma.adj_close_20ma,
	ma.adj_close_50ma,
	ma.adj_close_200ma,
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

--------------------------------------------------------------------------

-- =====================================================================
-- CRYPTO
-- =====================================================================

-- Crypto Prices
CREATE MATERIALIZED VIEW IF NOT EXISTS crypto.prices_matv AS
WITH 

-- moving averages for volume & price (adj. close), per 7, 90, and 365 x interval
moving_average_cte AS (
	SELECT
		pr.symbol_pk,
		pr.interval_pk,
		pr.source_pk,
		pr.dt,
		pr.volume,
		AVG(pr.volume) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk, pr.source_pk
			ORDER BY pr.dt
			ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
		) AS volume_7ma,
		AVG(pr.volume) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk, pr.source_pk
			ORDER BY pr.dt
			ROWS BETWEEN 89 PRECEDING AND CURRENT ROW
		) AS volume_90ma,
		AVG(pr.volume) OVER (
			PARTITION BY pr.symbol_pk, pr.interval_pk, pr.source_pk
			ORDER BY pr.dt
			ROWS BETWEEN 364 PRECEDING AND CURRENT ROW
		) AS volume_365ma
	FROM crypto.prices AS pr
),

-- price (adj. close) change percentages, per interval
percentage_change_cte AS (
	SELECT
		pr.symbol_pk,
		pr.interval_pk,
		pr.source_pk,
		pr.dt,
		pr.closing,
		CASE
			-- error case: division by zero
			WHEN LAG(pr.closing) OVER (
			    PARTITION BY pr.symbol_pk, pr.interval_pk, pr.source_pk
			    ORDER BY pr.dt
			) = 0 THEN NULL
			ELSE (pr.closing - LAG(pr.closing) OVER (
			    PARTITION BY pr.symbol_pk, pr.interval_pk, pr.source_pk
			    ORDER BY pr.dt
			)) / LAG(pr.closing) OVER (
			    PARTITION BY pr.symbol_pk, pr.interval_pk, pr.source_pk
			    ORDER BY pr.dt
			) * 100
		END AS perc
	FROM crypto.prices AS pr
)
SELECT
	sy.symbol,
	so.source,
	pr.dt,
	intv.interval,
	pc.perc,
	pr.opening,
	pr.high,
	pr.low,
	pr.closing,
	pr.volume,
	ma.volume_7ma,
	ma.volume_90ma,
	ma.volume_365ma,
	pr.trades
FROM crypto.symbols AS sy
INNER JOIN crypto.prices AS pr
	ON sy.pk = pr.symbol_pk
INNER JOIN crypto.sources AS so
	ON so.pk = pr.source_pk
INNER JOIN common.intervals AS intv
	ON intv.pk = pr.interval_pk

-- moving averages
LEFT JOIN moving_average_cte AS ma
	ON pr.symbol_pk = ma.symbol_pk
	AND pr.interval_pk = ma.interval_pk
	AND pr.source_pk = ma.source_pk
	AND pr.dt = ma.dt

-- percentage changes
LEFT JOIN percentage_change_cte AS pc
	ON pr.symbol_pk = pc.symbol_pk
	AND pr.interval_pk = pc.interval_pk
	AND pr.source_pk = pc.source_pk
	AND pr.dt = pc.dt
;

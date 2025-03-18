-- Provide additional metrics; transformations of source data,
-- e.g., moving averages, or percentage change
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.fact_prices (
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
	USING(symbol_pk)
INNER JOIN common.intervals AS intv
	USING(interval_pk)

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
);
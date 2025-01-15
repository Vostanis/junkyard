-- stock metrics 
CREATE MATERIALIZED VIEW IF NOT EXISTS stock.metrics_matv AS
SELECT
	sy.symbol,
	sy.title,
	m.dated,
	m.year,
	m.period,
	m.form,
	mlib.metric,
	m.val
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
SELECT
	sy.symbol,
	sy.title,
	iv.interval,
	pr.opening,
	pr.high,
	pr.low,
	pr.closing,
	pr.adj_close,
	pr.volume
FROM stock.symbols AS sy
INNER JOIN stock.prices AS pr
	ON sy.pk = pr.symbol_pk
INNER JOIN common.intervals AS iv
	ON iv.pk = pr.interval_pk
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

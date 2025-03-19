-- create quarterlies-only VIEW
DROP VIEW stock.quarterly_metrics;
CREATE VIEW stock.quarterly_metrics AS (
	SELECT
		symbol_pk,
		metric_pk,
		acc_pk,
		start_date,
		end_date,
		val,
		ARRAY_AGG(accn) AS accns,
		ARRAY_AGG(filing_date) AS filing_dates
	FROM stock.metrics
	WHERE
		(start_date IS NULL AND frame LIKE '%I')
		OR
		(end_date - start_date <= 100)
	GROUP BY
		symbol_pk,
		metric_pk,
		acc_pk,
		val,
		start_date,
		end_date
);
-- SELECT COUNT(*) FROM stock.quarterly_metrics; -- 38m rows
-- SELECT COUNT(*) FROM stock.metrics; -- 89m rows

-- create MATERIALIZED VIEW of std.financials
CREATE MATERIALIZED VIEW stock.fact_std_metrics
WITH
	revenue AS (
		SELECT
			-- *
			symbol_pk,
			start_date,
			end_date,
			val
		FROM stock.quarterly_metrics qm
		INNER JOIN stock.metrics_lib lib
			ON qm.metric_pk = lib.pk
		WHERE 
				lib.metric IN (
					'Revenues', 
					'RevenueFromContractWithCustomerExcludingAssessedTax', 
					'SalesRevenueNet'
				)
			AND qm.symbol_pk = 3 -- CASE: Nvidia
	)
SELECT DISTINCT
	qm.symbol_pk,
	qm.start_date,
	qm.end_date,
	revenue.val AS revenue
FROM stock.quarterly_metrics AS qm
LEFT JOIN revenue
	ON	qm.symbol_pk = revenue.symbol_pk
	AND	qm.start_date = revenue.start_date
	AND qm.end_date = revenue.end_date
WHERE qm.symbol_pk = 10546
ORDER BY end_date DESC
;
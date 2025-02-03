-- DELETE FROM stock.metrics
-- WHERE form = 'Inferred';

WITH
-- gather all the annual metrics and give an ID
annual_cte AS (
	SELECT DISTINCT
		ROW_NUMBER() OVER () AS id_a,
		symbol_pk,
		metric_pk,
		acc_pk,
		start_date,
		end_date,
		DATERANGE(start_date, end_date) AS date_range,
		val
	FROM stock.metrics
	WHERE 
			form = '10-K'
		AND start_date IS NOT NULL
		AND (end_date - start_date) BETWEEN 300 AND 380
	GROUP BY
		symbol_pk,
		metric_pk, 
		acc_pk,
		val,
		start_date,
		end_date
),

-- match all the quarterly metrics to an annual 1
quarterly_cte AS (
	SELECT DISTINCT
		id_a,
		-- give each date range an integer value
		-- (ideally, there would be 3 quarterly values per annual value)
		ROW_NUMBER() OVER (
			PARTITION BY id_a
			ORDER BY qs.start_date
		) AS id_q,
		qs.symbol_pk,
		qs.metric_pk,
		qs.acc_pk,
		qs.val,
		qs.start_date,
		qs.end_date,
		qs.date_range,
		anns.date_range AS matched_range
	FROM annual_cte anns

	-- inner join all the quarterly values
	INNER JOIN (
		SELECT DISTINCT
			symbol_pk,
			metric_pk,
			start_date,
			end_date,
			acc_pk,
			DATERANGE(start_date, end_date) AS date_range,
			MAX(val) AS val
		FROM stock.metrics
		WHERE
				form = '10-Q'
			AND start_date IS NOT NULL
			AND (end_date - start_date) <= 100 --  period length <= 100 days
		GROUP BY
			symbol_pk,
			metric_pk,
			acc_pk,
			start_date,
			end_date
	) qs ON 
			qs.date_range && anns.date_range
		AND qs.metric_pk = anns.metric_pk
		AND qs.symbol_pk = anns.symbol_pk
),

-- order the annual values alongside 3 matching quarterly values
ordered_cte AS (
	SELECT
		ann.symbol_pk,
		ann.metric_pk,
		ann.acc_pk,
		ann.val AS annual,
		MAX(qt.val) FILTER (WHERE qt.id_q = 1) AS q1,
		MAX(qt.val) FILTER (WHERE qt.id_q = 2) AS q2,
		MAX(qt.val) FILTER (WHERE qt.id_q = 3) AS q3,
		MAX(qt.val) FILTER (WHERE qt.id_q = 4) AS q4,
		ann.date_range,
		MAX(qt.start_date) FILTER (WHERE qt.id_q = 1) AS q1_start_date,
		MAX(qt.end_date) FILTER (WHERE qt.id_q = 1) AS q1_end_date,
		MAX(qt.start_date) FILTER (WHERE qt.id_q = 2) AS q2_start_date,
		MAX(qt.end_date) FILTER (WHERE qt.id_q = 2) AS q2_end_date,
		MAX(qt.start_date) FILTER (WHERE qt.id_q = 3) AS q3_start_date,
		MAX(qt.end_date) FILTER (WHERE qt.id_q = 3) AS q3_end_date,
		MAX(qt.start_date) FILTER (WHERE qt.id_q = 4) AS q4_start_date,
		MAX(qt.end_date) FILTER (WHERE qt.id_q = 4) AS q4_end_date
	FROM annual_cte ann
	INNER JOIN quarterly_cte qt
		ON qt.id_a = ann.id_a
		AND qt.symbol_pk = ann.symbol_pk
		AND qt.metric_pk = ann.metric_pk
	GROUP BY
		ann.symbol_pk,
		ann.metric_pk,
		ann.acc_pk,
		ann.val,
		ann.date_range
)

-- collect only inferred values; to be INSERT'd in to the original stock.metrics table
INSERT INTO stock.metrics (
	symbol_pk,
	metric_pk,
	acc_pk,
	start_date,
	end_date,
	filing_date,
	year,
	period,
	form,
	val,
	accn,
	frame
)
SELECT
	o.symbol_pk,
	o.metric_pk,
	o.acc_pk,
	
	-- infer "start_date"
	CASE
		-- q1
		WHEN (
			EXTRACT(MONTH FROM q1_start_date) NOT IN (1, 2, 3) AND
			EXTRACT(MONTH FROM q2_start_date) NOT IN (1, 2, 3) AND 
			EXTRACT(MONTH FROM q3_start_date) NOT IN (1, 2, 3)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_start_date)::INT, 1, 1)

		-- q2
		WHEN (
			EXTRACT(MONTH FROM q1_start_date) NOT IN (4, 5, 6) AND
			EXTRACT(MONTH FROM q2_start_date) NOT IN (4, 5, 6) AND 
			EXTRACT(MONTH FROM q3_start_date) NOT IN (4, 5, 6)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_start_date)::INT, 4, 1)

		-- q3
		WHEN (
			EXTRACT(MONTH FROM q1_start_date) NOT IN (7, 8, 9) AND
			EXTRACT(MONTH FROM q2_start_date) NOT IN (7, 8, 9) AND 
			EXTRACT(MONTH FROM q3_start_date) NOT IN (7, 8, 9)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_start_date)::INT, 7, 1)

		-- q4
		WHEN (
			EXTRACT(MONTH FROM q1_start_date) NOT IN (10, 11, 12) AND
			EXTRACT(MONTH FROM q2_start_date) NOT IN (10, 11, 12) AND 
			EXTRACT(MONTH FROM q3_start_date) NOT IN (10, 11, 12)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_start_date)::INT, 10, 1)
		ELSE '1970-01-01' 
	END AS start_date,
	
	-- infer "end_date"	
	CASE
		-- q1
		WHEN (
			EXTRACT(MONTH FROM q1_end_date) NOT IN (2, 3, 4) AND
			EXTRACT(MONTH FROM q2_end_date) NOT IN (2, 3, 4) AND 
			EXTRACT(MONTH FROM q3_end_date) NOT IN (2, 3, 4)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_end_date)::INT, 3, 31)

		-- q2
		WHEN (
			EXTRACT(MONTH FROM q1_end_date) NOT IN (5, 6, 7) AND
			EXTRACT(MONTH FROM q2_end_date) NOT IN (5, 6, 7) AND 
			EXTRACT(MONTH FROM q3_end_date) NOT IN (5, 6, 7)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_end_date)::INT, 6, 30)

		-- q3
		WHEN (
			EXTRACT(MONTH FROM q1_end_date) NOT IN (8, 9, 10) AND
			EXTRACT(MONTH FROM q2_end_date) NOT IN (8, 9, 10) AND 
			EXTRACT(MONTH FROM q3_end_date) NOT IN (8, 9, 10)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_end_date)::INT, 9, 30)

		-- q4
		WHEN (
			EXTRACT(MONTH FROM q1_end_date) NOT IN (11, 12, 1) AND
			EXTRACT(MONTH FROM q2_end_date) NOT IN (11, 12, 1) AND 
			EXTRACT(MONTH FROM q3_end_date) NOT IN (11, 12, 1)
		) THEN MAKE_DATE(EXTRACT(YEAR FROM q1_end_date)::INT, 12, 31)
		ELSE '1970-01-01'
	END AS end_date,
	
	'1970-01-01' AS filing_date,
	EXTRACT(YEAR FROM q1_end_date)::INT AS year,
	'I' AS period,
	'Inferred' AS form,
	o.annual - (o.q1 + o.q2 + o.q3) AS val,
	'Inferred' AS accn,
	NULL AS frame
FROM ordered_cte o
WHERE o.annual IS NOT NULL
	AND o.q1 IS NOT NULL
	AND o.q2 IS NOT NULL
	AND o.q3 IS NOT NULL
	AND o.q4 IS NULL
;

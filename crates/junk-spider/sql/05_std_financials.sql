CREATE MATERIALIZED VIEW stock.std_financials AS (
WITH
revenue AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		MAX(val) AS val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric IN (
			'Revenues', 
			'SalesRevenueNet', 
			'RevenueFromContractWithCustomerExcludingAssessedTax'
		)
	GROUP BY
		symbol_pk,
		start_date,
		end_date
),
gross_profit AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		MAX(val) AS val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric IN ('ProfitLoss', 'GrossProfit')
	GROUP BY
		symbol_pk,
		start_date,
		end_date
),
operating_income AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		MAX(val) AS val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric = 'OperatingIncomeLoss'
	GROUP BY
		symbol_pk,
		start_date,
		end_date
),
earnings AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		MAX(val) AS val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric = 'NetIncomeLoss'
	GROUP BY
		symbol_pk,
		start_date,
		end_date
),
avg_shares AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		MAX(val) AS val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric = 'WeightedAverageNumberOfSharesOutstandingBasic'
		AND period <> 'I'
	GROUP BY
		symbol_pk,
		start_date,
		end_date
),
accumulated_earnings AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric = 'RetainedEarningsAccumulatedDeficit'
	GROUP BY
		symbol_pk,
		start_date,
		end_date,
		val
),
debt AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		SUM(val) AS val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric IN (
			'LongTermDebtCurrent', 
			'LongTermDebtNoncurrent'
		)
	GROUP BY
		symbol_pk,
		start_date,
		end_date
),
equity AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE 	
		lib.metric = 'StockholdersEquity'
	GROUP BY
		symbol_pk,
		start_date,
		end_date,
		val
),

-- ASSETS
-- ==============================================================
assets AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'Assets'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
assets_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'AssetsCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
assets_non_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'AssetsNoncurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
cash AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'CashAndCashEquivalentsAtCarryingValue'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
marketable_securities_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'MarketableSecuritiesCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
nontrade_receivable_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'NontradeReceivablesCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
nontrade_receivable_non_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'NontradeReceivablesNoncurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
inventory_net AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'InventoryNet'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
property_plant_and_equipment_net AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'PropertyPlantAndEquipmentNet'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
other_assets_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'OtherAssetsCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
other_assets_non_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'OtherAssetsNoncurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
accounts_receivable_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'AccountsReceivableNetCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
-- ==============================================================

-- Liabilities
-- ==============================================================

liabilities AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'Liabilities'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
liabilities_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'LiabilitiesCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
liabilities_non_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'LiabilitiesNoncurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
accountables_payable_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'AccountsPayableCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
contracts_with_customer_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'ContractWithCustomerLiabilityCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
contracts_with_customer_non_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'ContractWithCustomerLiabilityNoncurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
commercial_paper AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'CommercialPaper'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
long_term_debt_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'LongTermDebtCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
long_term_debt_non_current AS (
	SELECT DISTINCT
		symbol_pk,
,
other_liabilities_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'OtherLiabilitiesCurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
other_liabilities_non_current AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'OtherLiabilitiesNoncurrent'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
-- ==============================================================

float AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'EntityPublicFloat'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
shares_outstanding AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'EntityCommonStockSharesOutstanding'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
shares_bought_back AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric IN ('StockRepurchasedAndRetiredDuringPeriodValue', 'StockRepurchasedDuringPeriodValue')
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
),
dividend_payout AS (
	SELECT DISTINCT
		symbol_pk,
		start_date,
		end_date,
		val
	FROM stock.metrics_q m
	INNER JOIN stock.metrics_lib lib
		ON m.metric_pk = lib.pk
	WHERE
		lib.metric = 'PaymentsOfDividends'
	GROUP BY 
		symbol_pk,
		start_date,
		end_date,
		val
)

SELECT DISTINCT
	m.symbol_pk,
	m.end_date,
	prices.adj_close 				AS price,
	shares_outstanding.val 			AS shares_outstanding,
	prices.adj_close
	* shares_outstanding.val		AS market_cap,
	revenue.val 					AS revenue,
	gross_profit.val				AS gross_profit,
	operating_income.val			AS operating_income,
	earnings.val 					AS earnings,
	CASE
		WHEN revenue.val <> 0 THEN earnings.val / revenue.val 		
		ELSE 0
	END AS earnings_perc,
	avg_shares.val					AS avg_shares,
	CASE
		WHEN avg_shares.val <> 0 THEN earnings.val / avg_shares.val
		ELSE 0
	END AS eps,
	accumulated_earnings.val 		AS accumulated_earnings,
	debt.val 						AS debt,
	equity.val 						AS equity,
	CASE
		WHEN equity.val <> 0 THEN earnings.val / equity.val
		ELSE 0
	END AS return_on_equity,
	CASE
		WHEN equity.val <> 0 THEN debt.val / equity.val
		ELSE 0
	END AS debt_to_equity,
	assets.val 						AS assets,
	CASE
		WHEN assets.val <> 0 THEN earnings.val / assets.val
		ELSE 0
	END AS return_on_assets,
	float.val 						AS float,
	shares_bought_back.val 			AS value_of_shares_bought_back,
	dividend_payout.val 			AS dividend_payout,

	-- ASSETS
	-- =======================
	assets_current.val AS assets_current,
	assets_non_current.val AS assets_non_current,
	cash.val AS cash,
	marketable_securities_current.val AS marketable_securities_current,
	nontrade_receivable_current.val AS nontrade_receivable_current,
	nontrade_receivable_non_current.val AS nontrade_receivable_non_current,
	inventory_net.val AS inventory_net,
	property_plant_and_equipment_net.val AS property_plant_and_equipment_net,
	other_assets_current.val AS other_assets_current,
	other_assets_non_current.val AS other_assets_non_current,
	accounts_receivable_current.val AS accounts_receivable_current,

	-- LIABILITIES
	-- =======================
	liabilities_current.val AS liabilities_current,
	liabilities_non_current.val AS liabilities_non_current,
	accounts_payable_current.val AS accounts_payable_current,
	contracts_with_customer_current.val AS contracts_with_customer_current,
	contracts_with_customer_non_current.val AS contracts_with_customer_non_current,
	commercial_paper.val AS commercial_paper,
	long_term_debt_current.val AS long_term_debt_current,
	long_term_debt_non_current.val AS long_term_debt_non_current,
	other_liabilities_current.val AS other_assets_current,
	other_liabilities_non_current.val AS other_liabilities_non_current,

FROM stock.metrics_q m
	LEFT JOIN stock.prices prices
		ON 	m.symbol_pk = prices.symbol_pk
		AND m.end_date = prices.dt::DATE
	LEFT JOIN revenue
		ON	m.symbol_pk = revenue.symbol_pk
		AND	m.end_date = revenue.end_date
	LEFT JOIN gross_profit
		ON	m.symbol_pk = gross_profit.symbol_pk
		AND	m.end_date = gross_profit.end_date
	LEFT JOIN operating_income
		ON	m.symbol_pk = operating_income.symbol_pk
		AND	m.end_date = operating_income.end_date
	LEFT JOIN earnings
		ON	m.symbol_pk = earnings.symbol_pk
		AND	m.end_date = earnings.end_date
	LEFT JOIN avg_shares
		ON	m.symbol_pk = avg_shares.symbol_pk
		AND	m.end_date = avg_shares.end_date
	LEFT JOIN accumulated_earnings
		ON	m.symbol_pk = accumulated_earnings.symbol_pk
		AND	m.end_date = accumulated_earnings.end_date
	LEFT JOIN debt
		ON	m.symbol_pk = debt.symbol_pk
		AND	m.end_date = debt.end_date
	LEFT JOIN equity
		ON	m.symbol_pk = equity.symbol_pk
		AND	m.end_date = equity.end_date

	-- ASSETS
	-- ===============================================
	LEFT JOIN assets
		ON	m.symbol_pk = assets.symbol_pk
		AND	m.end_date = assets.end_date
	LEFT JOIN assets_current
		ON	m.symbol_pk = assets_current.symbol_pk
		AND	m.end_date = assets_current.end_date
	LEFT JOIN assets_non_current
		ON	m.symbol_pk = assets_non_current.symbol_pk
		AND	m.end_date = assets_non_current.end_date
	LEFT JOIN cash
		ON	m.symbol_pk = cash.symbol_pk
		AND	m.end_date = cash.end_date
	LEFT JOIN marketable_securities_current
		ON	m.symbol_pk = marketable_securities_current.symbol_pk
		AND	m.end_date = marketable_securities_current.end_date
	LEFT JOIN nontrade_receivable_current
		ON	m.symbol_pk = nontrade_receivable_current.symbol_pk
		AND	m.end_date = nontrade_receivable_current.end_date
	LEFT JOIN nontrade_receivable_non_current
		ON	m.symbol_pk = nontrade_receivable_non_current.symbol_pk
		AND	m.end_date = nontrade_receivable_non_current.end_date
	LEFT JOIN inventory_net
		ON	m.symbol_pk = inventory_net.symbol_pk
		AND	m.end_date = inventory_net.end_date
	LEFT JOIN property_plant_and_equipment_net
		ON	m.symbol_pk = property_plant_and_equipment_net.symbol_pk
		AND	m.end_date = property_plant_and_equipment_net.end_date
	LEFT JOIN other_assets_current
		ON	m.symbol_pk = other_assets_current.symbol_pk
		AND	m.end_date = other_assets_current.end_date
	LEFT JOIN other_assets_non_current
		ON	m.symbol_pk = other_assets_non_current.symbol_pk
		AND	m.end_date = other_assets_non_current.end_date
	LEFT JOIN accounts_receivable_current
		ON	m.symbol_pk = accounts_receivable_current.symbol_pk
		AND	m.end_date = accounts_receivable_current.end_date

	-- LIABILITIES
	-- ==============================================
	LEFT JOIN liabilities
		ON	m.symbol_pk = liabilities.symbol_pk
		AND	m.end_date = liabilities.end_date
	LEFT JOIN liabilities_current
		ON	m.symbol_pk = liabilities_current.symbol_pk
		AND	m.end_date = liabilities_current.end_date
	LEFT JOIN liabilities_non_current
		ON	m.symbol_pk = liabilities_non_current.symbol_pk
		AND	m.end_date = liabilities_non_current.end_date
	LEFT JOIN accounts_payable_current
		ON	m.symbol_pk = accounts_payable_current.symbol_pk
		AND	m.end_date = accounts_payable_current.end_date
	LEFT JOIN contracts_with_customer_current
		ON	m.symbol_pk = contracts_with_customer_current.symbol_pk
		AND	m.end_date = contracts_with_customer_current.end_date
	LEFT JOIN contracts_with_customer_non_current
		ON	m.symbol_pk = contracts_with_customer_non_current.symbol_pk
		AND	m.end_date = contracts_with_customer_non_current.end_date
	left join commercial_paper
		on	m.symbol_pk = commercial_paper.symbol_pk
		and	m.end_date = commercial_paper.end_date
	left join long_term_debt_current
		on	m.symbol_pk = long_term_debt_current.symbol_pk
		and	m.end_date = long_term_debt_current.end_date
	left join long_term_debt_non_current
		on	m.symbol_pk = long_term_debt_non_current.symbol_pk
		and	m.end_date = long_term_debt_non_current.end_date
	left join other_liabilities_current
		on	m.symbol_pk = other_liabilities_current.symbol_pk
		and	m.end_date = other_liabilities_current.end_date
	left join other_liabilities_non_current
		on	m.symbol_pk = other_liabilities_non_current.symbol_pk
		and	m.end_date = other_liabilities_non_current.end_date

	-- MARKET MECHANICS
	-- ==============================================
	LEFT JOIN float
		ON	m.symbol_pk = float.symbol_pk
		AND	m.end_date = float.end_date
	LEFT JOIN shares_outstanding
		ON	m.symbol_pk = shares_outstanding.symbol_pk
		AND	m.end_date = shares_outstanding.end_date
	LEFT JOIN shares_bought_back
		ON	m.symbol_pk = shares_bought_back.symbol_pk
		AND 	m.end_date = shares_bought_back.end_date
	LEFT JOIN dividend_payout
		ON	m.symbol_pk = dividend_payout.symbol_pk
		AND 	m.end_date = dividend_payout.end_date
		
-- WHERE
-- 	m.symbol_pk = (SELECT symbol_pk FROM const)
);

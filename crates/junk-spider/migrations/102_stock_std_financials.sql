CREATE MATERIALIZED VIEW stock.std_financials AS (
  WITH 
 	-- prices AS (),
	-- metrics_that_are_not_quarterly AS (),
  	metrics_pivoted AS (
    -- Pivot all metrics from stock.metrics_q and stock.metrics in one pass
    SELECT
      m.symbol_pk,
      m.start_date,
      m.end_date,
      MAX(CASE WHEN lib.metric IN ('Revenues', 'SalesRevenueNet', 'RevenueFromContractWithCustomerExcludingAssessedTax') THEN m.val END) AS revenue,
      MAX(CASE WHEN lib.metric IN ('ProfitLoss', 'GrossProfit') THEN m.val END) AS gross_profit,
      MAX(CASE WHEN lib.metric = 'OperatingIncomeLoss' THEN m.val END) AS operating_income,
      MAX(CASE WHEN lib.metric = 'NetIncomeLoss' THEN m.val END) AS earnings,
      MAX(CASE WHEN lib.metric = 'WeightedAverageNumberOfSharesOutstandingBasic' AND m.period <> 'I' THEN m.val END) AS avg_shares,
      MAX(CASE WHEN lib.metric = 'RetainedEarningsAccumulatedDeficit' THEN m.val END) AS accumulated_earnings,
      SUM(CASE WHEN lib.metric IN ('LongTermDebtCurrent', 'LongTermDebtNoncurrent') THEN m.val ELSE 0 END) AS debt,
      MAX(CASE WHEN lib.metric = 'StockholdersEquity' THEN m.val END) AS equity,
      MAX(CASE WHEN lib.metric = 'Assets' THEN m.val END) AS assets,
      MAX(CASE WHEN lib.metric = 'AssetsCurrent' THEN m.val END) AS assets_current,
      MAX(CASE WHEN lib.metric = 'AssetsNoncurrent' THEN m.val END) AS assets_non_current,
      MAX(CASE WHEN lib.metric = 'CashAndCashEquivalentsAtCarryingValue' THEN m.val END) AS cash,
      MAX(CASE WHEN lib.metric = 'MarketableSecuritiesCurrent' THEN m.val END) AS marketable_securities_current,
      MAX(CASE WHEN lib.metric = 'NontradeReceivablesCurrent' THEN m.val END) AS nontrade_receivable_current,
      MAX(CASE WHEN lib.metric = 'NontradeReceivablesNoncurrent' THEN m.val END) AS nontrade_receivable_non_current,
      MAX(CASE WHEN lib.metric = 'InventoryNet' THEN m.val END) AS inventory_net,
      MAX(CASE WHEN lib.metric = 'PropertyPlantAndEquipmentNet' THEN m.val END) AS property_plant_and_equipment_net,
      MAX(CASE WHEN lib.metric = 'OtherAssetsCurrent' THEN m.val END) AS other_assets_current,
      MAX(CASE WHEN lib.metric = 'OtherAssetsNoncurrent' THEN m.val END) AS other_assets_non_current,
      MAX(CASE WHEN lib.metric = 'AccountsReceivableNetCurrent' THEN m.val END) AS accounts_receivable_current,
      MAX(CASE WHEN lib.metric = 'Liabilities' THEN m.val END) AS liabilities,
      MAX(CASE WHEN lib.metric = 'LiabilitiesCurrent' THEN m.val END) AS liabilities_current,
      MAX(CASE WHEN lib.metric = 'LiabilitiesNoncurrent' THEN m.val END) AS liabilities_non_current,
      MAX(CASE WHEN lib.metric = 'AccountsPayableCurrent' THEN m.val END) AS accounts_payable_current,
      MAX(CASE WHEN lib.metric = 'ContractWithCustomerLiabilityCurrent' THEN m.val END) AS contracts_with_customer_current,
      MAX(CASE WHEN lib.metric = 'ContractWithCustomerLiabilityNoncurrent' THEN m.val END) AS contracts_with_customer_non_current,
      MAX(CASE WHEN lib.metric = 'CommercialPaper' THEN m.val END) AS commercial_paper,
      MAX(CASE WHEN lib.metric = 'LongTermDebtCurrent' THEN m.val END) AS long_term_debt_current,
      MAX(CASE WHEN lib.metric = 'LongTermDebtNoncurrent' THEN m.val END) AS long_term_debt_non_current,
      MAX(CASE WHEN lib.metric = 'OtherLiabilitiesCurrent' THEN m.val END) AS other_liabilities_current,
      MAX(CASE WHEN lib.metric = 'OtherLiabilitiesNoncurrent' THEN m.val END) AS other_liabilities_non_current,
      MAX(CASE WHEN lib.metric = 'EntityPublicFloat' THEN m.val END) AS float,
      MAX(CASE WHEN lib.metric = 'EntityCommonStockSharesOutstanding' THEN m.val END) AS shares_outstanding,
      MAX(CASE WHEN lib.metric IN ('StockRepurchasedAndRetiredDuringPeriodValue', 'StockRepurchasedDuringPeriodValue') THEN m.val END) AS shares_bought_back,
      MAX(CASE WHEN lib.metric = 'PaymentsOfDividends' THEN m.val END) AS dividend_payout
    FROM stock.metrics_q m
    INNER JOIN stock.metrics_lib lib ON m.metric_pk = lib.pk
    GROUP BY m.symbol_pk, m.start_date, m.end_date
  )
  SELECT
    m.symbol_pk,
    m.end_date,
    p.adj_close AS price,
    m.shares_outstanding,
    p.adj_close * m.shares_outstanding AS market_cap,
    m.revenue,
    m.gross_profit,
    m.operating_income,
    m.earnings,
    CASE WHEN m.revenue <> 0 THEN m.earnings / m.revenue ELSE 0 END AS earnings_perc,
    m.avg_shares,
    CASE WHEN m.avg_shares <> 0 THEN m.earnings / m.avg_shares ELSE 0 END AS eps,
    m.accumulated_earnings,
    m.debt,
    m.equity,
    CASE WHEN m.equity <> 0 THEN m.earnings / m.equity ELSE 0 END AS return_on_equity,
    CASE WHEN m.equity <> 0 THEN m.debt / m.equity ELSE 0 END AS debt_to_equity,
    m.assets,
    CASE WHEN m.assets <> 0 THEN m.earnings / m.assets ELSE 0 END AS return_on_assets,
    m.float,
    m.shares_bought_back AS value_of_shares_bought_back,
    m.dividend_payout,

    -- Assets
    m.assets_current,
    m.assets_non_current,
    m.cash,
    m.marketable_securities_current,
    m.nontrade_receivable_current,
    m.nontrade_receivable_non_current,
    m.inventory_net,
    m.property_plant_and_equipment_net,
    m.other_assets_current,
    m.other_assets_non_current,
    m.accounts_receivable_current,

    -- Liabilities
    m.liabilities_current,
    m.liabilities_non_current,
    m.accounts_payable_current,
    m.contracts_with_customer_current,
    m.contracts_with_customer_non_current,
    m.commercial_paper,
    m.long_term_debt_current,
    m.long_term_debt_non_current,
    m.other_liabilities_current,
    m.other_liabilities_non_current
  FROM metrics_pivoted m
  LEFT JOIN stock.prices p
    ON m.symbol_pk = p.symbol_pk
    AND m.end_date = p.dt::DATE
);


SELECT * FROM stock.std_financials LIMIT 50

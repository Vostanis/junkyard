# Blueprint

Only encompassing US stocks so far, the scraper collects fundamental Metrics data, alongside Price data.

-----------------------------------------------------------------------------------------

## SEC (Security & Exchange Commission)

Every US stock files quarterly & annual reports here. Endpoints of interest:

1. `https://www.sec.gov/files/company_tickers.json` for a list of US Companies;
2. `https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip` for all filed reporting metrics (Revenue, EPS, etc.);
3. `https://www.sec.gov/archives/edgar/daily-index/bulkdata/submissions.zip` for all submission metadata (and a few extra details, such as SIC code).

-----------------------------------------------------------------------------------------

## Yahoo! Finance

Yahoo provides almost unlimited access to data from their endpoints, and the prices endpoint is as follows:

`https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?symbol={ticker}&interval={interval}&range={range}&events=div|split|capitalGains`

An example could be:
ticker = `NVDA`
interval = `1d`
range = `10y`

-----------------------------------------------------------------------------------------

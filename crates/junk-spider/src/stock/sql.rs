#![allow(dead_code)]

//////////////////////////////////////////////////////////////////
// tickers
//////////////////////////////////////////////////////////////////
pub(crate) static INSERT_TICKER: &'static str = "
    INSERT INTO stock.tickers (pk, cik, ticker, title, industry, nation)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (pk) DO NOTHING
";

//////////////////////////////////////////////////////////////////
// prices
//////////////////////////////////////////////////////////////////
pub(crate) static INSERT_PRICE: &'static str = "
    INSERT INTO stock.prices (stock_pk, time, interval_pk, opening, high, low, closing, adj_close, volume)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ON CONFLICT (stock_id, time, interval) DO NOTHING
";

//////////////////////////////////////////////////////////////////
// metrics
//////////////////////////////////////////////////////////////////
pub(crate) static INSERT_METRIC: &'static str = "
    INSERT INTO stock.metrics (stock_pk, dated, metric, val, unit, taxonomy)
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (stock_id, dated, metric, val, unit, taxonomy) DO NOTHING
";

//////////////////////////////////////////////////////////////////
// filings
//////////////////////////////////////////////////////////////////
pub(crate) static INSERT_FILING: &'static str = "
    INSERT INTO stock.filings (stock_id, dated, filename, filetype, url, content, content_ts)
    VALUES ($1, $2, $3, $4, $5, $6, to_tsvector($6))
    ON CONFLICT (stock_id, filename) DO NOTHING
";

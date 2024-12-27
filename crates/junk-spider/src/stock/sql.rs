#![allow(dead_code)]

use std::collections::HashMap;

//////////////////////////////////////////////////////////////////
// tickers
//////////////////////////////////////////////////////////////////

/// `stock.tickers` is the master table for stock tickers, their title, industry labels, and their
/// nation.
pub(crate) static INSERT_TICKER: &'static str = "
    INSERT INTO stock.tickers (cik, ticker, title, industry, nation)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (pk) DO NOTHING
";

//////////////////////////////////////////////////////////////////
// prices
//////////////////////////////////////////////////////////////////

/// `stock.prices` is the master table for stock prices.
pub(crate) static INSERT_PRICE: &'static str = "
    INSERT INTO stock.prices (symbol_pk, dt, interval_pk, opening, high, low, closing, adj_close, volume)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ON CONFLICT (symbol_pk, dt, interval_pk) DO NOTHING
";

//////////////////////////////////////////////////////////////////
// metrics
//////////////////////////////////////////////////////////////////

/// `stock.metrics` is an opimtised master table for fundamental metrics, related to stocks.
/// Examples include "Revenues", "Net Income", "Total Assets", etc.
pub(crate) static INSERT_METRIC: &'static str = "
    INSERT INTO stock.metrics (symbol_pk, metric_pk, acc_pk, dated, val)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (stock_id, dated, metric, val, unit, taxonomy) DO NOTHING
";

/// When a new metric is found (when scraping the SEC's compfanyfacts.zip), insert it into
/// the metrics library, giving it a Primary Key.
pub(crate) static INSERT_METRIC_PK: &'static str = "
    INSERT INTO stock.metrics_lib (metric)
    VALUES ($1)
    ON CONFILCT (pk) DO NOTHING
";

pub(crate) static GET_METRIC_PK: &'static str = "
    SELECT pk 
    FROM stock.metrics 
    WHERE metric = ($1)
";

lazy_static::lazy_static! {
    /// Static Primary Key table helps streamline the code when scraping `stock.metrics`.
    pub(crate) static ref ACC_STDS: HashMap<String, i32> = {
        let mut map = HashMap::new();
        map.insert("us-gaap".to_string(), 1);
        map.insert("dei".to_string(), 2);
        map.insert("srt".to_string(), 3);
        map.insert("invest".to_string(), 4);
        map
    };
}

/// Each metric is under a different set of Accounting Standards; the acronyms of which
/// are stored in `stock.metrics_acc`, and given a Primary Key.
pub(crate) static INSERT_METRIC_ACC_PK: &'static str = "
    INSERT INTO stock.acc_stds (acc)
    VALUES ($1)
    ON CONFLICT (pk) DO NOTHING
";

//////////////////////////////////////////////////////////////////
// filings
//////////////////////////////////////////////////////////////////
pub(crate) static INSERT_FILING: &'static str = "
    INSERT INTO stock.filings (stock_id, dated, filename, filetype, url, content, content_ts)
    VALUES ($1, $2, $3, $4, $5, $6, to_tsvector($6))
    ON CONFLICT (stock_id, filename) DO NOTHING
";

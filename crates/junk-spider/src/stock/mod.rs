mod sql;

pub(crate) mod common;

/// US stock information from the [SEC]; all tickers, titles and industries, as well as any metric & filings data.
///
/// [SEC]: https://www.sec.gov/search-filings/edgar-application-programming-interfaces
pub mod sec;

/// Price data collected from the Yahoo Finance API; inspiration from Python's [yfinance] library.
///
/// [yfinance]: https://github.com/ranaroussi/yfinance/
pub mod yahoo_finance;

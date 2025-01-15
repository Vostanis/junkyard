//////////////////////////////////////////////////////////////////////
///
/// Data collection libraries
///
//////////////////////////////////////////////////////////////////////

/// Cryptocurrency data, collected from the REST APIs of various exchanges.
///
/// Examples include **Binance, KuCoin, MEXC, Kraken**.
pub mod crypto;

/// Economic data;
///
/// - US data collected from [FRED](https://fred.stlouisfed.org/docs/api/fred/).
pub mod econ;

/// Stock data, collected from various sources.
///
/// Examples include **Yahoo! Finance & the SEC**.
pub mod stock;

//////////////////////////////////////////////////////////////////////
///
/// Utilities
///
//////////////////////////////////////////////////////////////////////

/// Colored logging function.
pub(crate) fn time_elapsed(time: std::time::Instant) -> String {
    use colored::Colorize;
    format!("< Time elapsed: {} ms >", time.elapsed().as_millis())
        .truecolor(224, 60, 138)
        .to_string()
}

/// Standard client build for HTTP requests, only requiring a User-Agent Environrment Variable.
pub(crate) fn std_client_build() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(&dotenv::var("USER_AGENT").expect("failed to read USER_AGENT"))
        .build()
        .expect("failed to build reqwest::Client")
}

/// Shortcuts used in HTTP API requests.
pub mod http {
    pub use dotenv::var;
    pub use reqwest::Client as HttpClient;
    pub use tokio_postgres::Client as PgClient;
}

/// File store functions.
pub mod fs;

/// The [KeyTracker](key_tracker::KeyTracker) struct is used to track the state of available primary keys.
pub mod key_tracker;

/// Common TUI functions.
pub mod tui;

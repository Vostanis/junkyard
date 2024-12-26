//////////////////////////////////////////////////////////////////////
///
/// Data collection libraries
///
//////////////////////////////////////////////////////////////////////

/// Cryptocurrency data collection.
///
/// Examples include **Binance, KuCoin, MEXC, Kraken**.
pub mod crypto;

// pub mod econ;

/// Stock data collection.
///
/// Examples include **Yahoo! Finance & the SEC**.
pub mod stock;

//////////////////////////////////////////////////////////////////////
///
/// Utilities
///
//////////////////////////////////////////////////////////////////////

/// Shortcuts used in HTTP API requests.
pub mod http {
    pub use dotenv::var;
    pub use reqwest::Client as HttpClient;
    pub use tokio_postgres::Client as PgClient;
}

/// File store functions.
pub mod fs;

use anyhow::Result;
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as HttpClient, ClientBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use tokio_postgres::Client as PgClient;
use tracing::{debug, error, info};

/// Represents exchange-specific data types that can be fetched from HTTP
/// and loaded into the database.
pub trait ExchangeData: for<'de> Deserialize<'de> {
    /// The API endpoint to fetch this data
    fn url(&self, symbol: Option<&str>, interval: Option<&str>) -> String;

    /// Load this data into the database
    async fn pg_load(
        &self,
        pool: &Pool,
        metadata: Option<&ExchangeMetadata>,
    ) -> Result<()>;
}

/// Metadata needed for database operations.
pub struct ExchangeMetadata {
    pub source_pk: i16,
    pub symbol_pks: HashMap<String, i32>,
    pub interval_pks: HashMap<String, i16>,
}

/// Core trait for crypto exchange brokers.
pub trait Broker {
    /// Name of the exchange.
    fn name() -> &'static str;

    /// HTTP headers required for API authentication.
    fn http_headers() -> Vec<(&'static str, String)>;

    /// Available trading intervals (e.g., "1h", "1d").
    fn intervals() -> Vec<&'static str> {
        vec!["1h", "1d", "1w"]
    }

    /// Fetch ticker symbols.
    async fn fetch_symbols(&self, client: &HttpClient) -> Result<impl ExchangeData>;

    /// Fetch price data for a symbol and interval.
    async fn fetch_prices(
        &self,
        client: &HttpClient,
        symbol: &str,
        interval: &str,
    ) -> Result<impl ExchangeData>;

    /// Build an HTTP client with the required headers.
    fn build_http_client() -> Result<HttpClient> {
        let headers = Self::http_headers();
        let mut header_map = HeaderMap::new();

        for (label, value) in headers.iter() {
            let header_value = HeaderValue::from_str(value).map_err(|e| {
                error!("Failed to create header value for {}: {}", label, e);
                e
            })?;
            header_map.insert(*label, header_value);
        }

        let client = ClientBuilder::new()
            .default_headers(header_map)
            .build()
            .map_err(|e| {
                error!("Failed to build HTTP client for {}: {}", Self::name(), e);
                e
            })?;

        Ok(client)
    }

    /// Generalised framework for the full API-webscraping process.
    async fn execute(&self, pool: &Pool) -> Result<()> {
        // Execute the entire data collection process.
        info!("Starting data collection for {}", Self::name());

        // Build HTTP client.
        let http_client = Self::build_http_client()?;

        // Fetch symbols.
        info!("Fetching symbols from {}", Self::name());
        let symbols = self.fetch_symbols(&http_client).await?;
        symbols.pg_load(pool, None).await?;

        // Load symbols into database.
        info!("Loading symbols into database");
        let source_pk = super::common::existing_source(pool, Self::name().to_string()).await?;
        let symbol_pks = super::common::existing_symbols(pool).await?;
        let interval_pks = super::common::existing_intervals(pool).await?;
        let metadata = ExchangeMetadata {
            source_pk,
            symbol_pks,
            interval_pks,
        };

        // Process each symbol and interval combination.
        info!("Fetching price data for {} symbols", symbols.len());
        let mut success_count = 0;
        let mut error_count = 0;

        let intervals = Self::intervals();
        for interval in intervals {
            let mut stream = stream::iter(symbols);
            while let Some(symbol) = symbols.next().await {
                let = http_client = &http_client;

                async move {}.await;
            }
        }

        info!(
            "Completed data collection for {}: {} successful, {} failed",
            Self::name(),
            success_count,
            error_count
        );

        Ok(())
    }
}

// Implementations for specific brokers would derive from this trait


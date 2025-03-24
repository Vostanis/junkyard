// use crate::if_tui;
use super::common;
// use crate::util::num_concurrent_threads;

use anyhow::{anyhow, Result};
use dotenv::var;
use futures::{stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder};
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use sqlx::PgPool;
use tokio_postgres::Client as PgClient;
use tracing::{debug, error, info, trace};

/// Constant used in debugging messages.
const DEBUG_API: &str = "Binance";

/// Client for Binance requires the "X-MBX-APIKEY" header.
pub(super) fn client() -> Result<Client> {
    let env_var = "BINANCE_API";
    let api_key = var(env_var)?;

    if api_key.is_empty() {
        return Err(anyhow!("API key is empty - {env_var} required"));
    }

    let mut headers = HeaderMap::new();
    let api_header = HeaderValue::from_str(&api_key)?;
    headers.insert("X-MBX-APIKEY", api_header);
    let client = ClientBuilder::new().default_headers(headers).build()?;

    Ok(client)
}

/// Ticker symbol datatype and custom database-loading, for the following:
///
/// ```json
/// [
///     {
///         "symbol": "ETHBTC",
///         "bidPrice": "0.03699000",
///         "bidQty": "39.03480000",
///         "askPrice": "0.03700000",
///         "askQty": "7.12410000"
///     },
///     {
///         "symbol": "LTCBTC",
///         "bidPrice": "0.00119000",
///         "bidQty": "134.49500000",
///         "askPrice": "0.00119100",
///         "askQty": "58.48100000"
///     },
/// ]
/// ```
#[derive(Debug, Deserialize)]
pub(super) struct Tickers(pub(super) Vec<Ticker>);

#[derive(Debug, Deserialize)]
pub(super) struct Ticker {
    symbol: String,
}

impl Tickers {
    async fn insert(&self, client: &mut PgClient) -> Result<()> {
        // Start a transaction with a prepared statement.
        let stmt = client.prepare(common::INSERT_SYMBOL).await?;
        let tx = client.transaction().await?;

        // Stream the symbols & insert them to the database.
        let mut stream = stream::iter(&self.0);
        while let Some(ticker) = stream.next().await {
            let stmt = &stmt;
            let tx = &tx;
            async move {
                match tx.execute(stmt, &[&ticker.symbol]).await {
                    Ok(_) => trace!("{stmt:#?} executed successfully for {}", &ticker.symbol),
                    Err(e) => error!("Failed to execute {stmt:#?} for {}: {e}", &ticker.symbol),
                };
            }
            .await;
        }

        // Commit the transaction.
        tx.commit().await?;

        Ok(())
    }
}

/// Price datatype, per ticker symbol, and custom deserialization, for the following:
///
/// ```json
///
/// [
///   [
///     1499040000000,      // Kline open time
///     "0.01634790",       // Open price
///     "0.80000000",       // High price
///     "0.01575800",       // Low price
///     "0.01577100",       // Close price
///     "148976.11427815",  // Volume
///     1499644799999,      // Kline Close time
///     "2434.19055334",    // Quote asset volume
///     308,                // Number of trades
///     "1756.87402397",    // Taker buy base asset volum.e
///     "28.46694368",      // Taker buy quote asset volume
///     "0"                 // Unused field, ignore.
///   ],
/// ]
/// ```
#[derive(Debug, Deserialize)]
struct Klines(Vec<Kline>);

#[derive(Deserialize, Debug)]
struct Kline {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    _close_timestamp: IgnoredAny,
    _quote_asset_volume: IgnoredAny,
    trades: i64,
    _taker_buy_base_asset_volume: IgnoredAny,
    _taker_buy_quote_asset_volume: IgnoredAny,
    _unused: IgnoredAny,
}

impl<'de> Visitor<'de> for Kline {
    type Value = Kline;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Array of Klines")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok(Kline {
            timestamp: seq.next_element::<i64>()?.expect("i64 timestamp"),
            open: seq
                .next_element::<String>()?
                .expect("String open")
                .parse::<f64>()
                .expect("failed to transform String to f64"),
            high: seq
                .next_element::<String>()?
                .expect("String high")
                .parse::<f64>()
                .expect("failed to transform String to f64"),
            low: seq
                .next_element::<String>()?
                .expect("String low")
                .parse::<f64>()
                .expect("failed to transform String to f64"),
            close: seq
                .next_element::<String>()?
                .expect("String close")
                .parse::<f64>()
                .expect("failed to transform String to f64"),
            volume: seq
                .next_element::<String>()?
                .expect("String volume")
                .parse::<f64>()
                .expect("failed to transform String to f64"),
            _close_timestamp: seq
                .next_element::<IgnoredAny>()?
                .expect("i64 close timestamp"),
            _quote_asset_volume: seq
                .next_element::<IgnoredAny>()?
                .expect("String quote asset volume"),
            trades: seq.next_element::<i64>()?.expect("i64 number of trades"),
            _taker_buy_base_asset_volume: seq
                .next_element::<IgnoredAny>()?
                .expect("String taker buy base asset volume"),
            _taker_buy_quote_asset_volume: seq
                .next_element::<IgnoredAny>()?
                .expect("String taker buy quote asset volume"),
            _unused: seq
                .next_element::<IgnoredAny>()?
                .expect("String unused (ignore this field)"),
        })
    }
}

impl Klines {
    /// Insert the Price data, having already fetched the required Primary Keys.
    async fn insert(
        &self,
        client: &mut PgClient,
        symbol_pk: i32,
        interval_pk: i32,
        source_pk: i32,
    ) -> Result<()> {
        // Start a transaction with a prepared statement.
        let stmt = client.prepare(common::INSERT_PRICE).await?;
        let tx = client.transaction().await?;

        // Stream the priceset & insert each cell to the database.
        let mut stream = stream::iter(&self.0);
        while let Some(price_cell) = stream.next().await {
            let stmt = &stmt;
            let tx = &tx;
            async move {
                match tx
                    .execute(
                        stmt,
                        &[
                            &symbol_pk,
                            // Transform the TIMESTAMP (ms) to a TIMESTAMP WITH TIMEZONE.
                            &chrono::DateTime::from_timestamp_millis(price_cell.timestamp)
                                .expect("Invalid timestamp (ms)"),
                            &interval_pk,
                            &price_cell.open,
                            &price_cell.high,
                            &price_cell.low,
                            &price_cell.close,
                            &price_cell.volume,
                            &price_cell.trades,
                            &source_pk,
                        ],
                    )
                    .await
                {
                    Ok(_) => trace!("{stmt:?} successfully executed for ID[{symbol_pk}]"),
                    Err(e) => error!("Failed to execute {stmt:?} for ID[{symbol_pk}]: {e}"),
                };
            }
            .await;
        }

        // Commit the transaction.
        tx.commit().await?;

        Ok(())
    }
}

/// Fetch the Ticker Symbol list.
pub(crate) async fn get_symbols() -> Result<Tickers> {
    // Build the HTTP client.
    let client = client().map_err(|e| {
        error!("Client failed to build: {e}");
        e
    })?;

    // Make the HTTP request, deserializing it straight away.
    let tickers: Tickers = client
        .get("https://api.binance.com/api/v1/ticker/allBookTickers")
        .send()
        .await
        .map_err(|e| {
            error!("Failed sending request for ticker symbols: {e}");
            e
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Failed deserializing ticker symbols: {e}");
            e
        })?;

    Ok(tickers)
}

/// Perform the entire webscraping process.
///
/// ## Information
///
/// 1. Binance has a RATE_LIMIT = 1200 per 60s.
///
/// 2. Endpoints:
///     a) Tickers = "https://api.binance.com/api/v1/ticker/allBookTickers"
///     b) Prices, per symbol = "https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1d&limit=1000"
///
/// ## Process
///
/// 1. Fetch & insert the all the Ticker Symbols.
///
/// 2. Fetch hashmaps of Primary Keys for 2 tables: Symbols & Sources.
///    This is done so we can insert the Primary Key, instead of the String.
///
/// 3. Webscraping the prices (asynchronously).
pub(crate) async fn webscrape(pool: &PgPool, tui: bool) -> Result<()> {
    // 1. Fetch & insert all the Ticker Symbols.
    if tui {
        println!("fetching tickers from {DEBUG_API} ...");
    }
    debug!("Requesting ticker symbols");
    let symbols = get_symbols().await?;
    trace!("{} ticker symbols retrieved", symbols.0.len());

    debug!("Inserting ticker symbols to database");
    symbols.insert(pool).await?;

    // 2. Fetch Hashmaps of Primary Keys for 2 tables: Symbols & Sources.
    if tui {
        println!("fetching existing Primary Keys ...");
    }
    debug!("Retrieving existing Symbol Primary Keys");
    let symbol_pks = super::common::existing_symbols(pool).await?;
    trace!(
        "{} existing symbol primary keys retrieved",
        symbol_pks.len()
    );

    debug!("Retrieving existing Source Primary Keys");
    let source_pks = super::common::existing_sources(pool).await?;
    trace!(
        "{} existing source primary keys retrieved",
        source_pks.len()
    );

    // 3. Async webscraping the prices.
    if tui {
        println!("webscraping prices from {DEBUG_API} ...");
    }

    Ok(())
}

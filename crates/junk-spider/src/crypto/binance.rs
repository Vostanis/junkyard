use super::sql;
use crate::http::*;
use futures::{stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::ClientBuilder;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

// RATE_LIMIT = 1200 /60s
//
// tickers = `https://api.binance.com/api/v1/ticker/allBookTickers`
//
// klines = `https://api.binance.com/api/v3/klines`, per symbol

/////////////////////////////////////////////////////////////////////////////////
// core
/////////////////////////////////////////////////////////////////////////////////

pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
    let http_client = build_client();

    // fetch the tickers
    let tickers: Tickers = http_client
        .get("https://api.binance.com/api/v1/ticker/allBookTickers")
        .send()
        .await
        .map_err(|err| {
            error!("failed to fetch Binance tickers");
            err
        })?
        .json()
        .await
        .map_err(|err| {
            error!("failed to dserialize Binance tickers");
            err
        })?;

    // 1. insert binance source
    pg_client
        .query(
            "INSERT INTO crypto.sources (source) VALUES ('Binance') ON CONFLICT DO NOTHING",
            &[],
        )
        .await
        .map_err(|err| {
            error!("failed to insert Binance as a source");
            err
        })?;

    // 2. insert tickers
    tickers.insert(pg_client).await?;

    // 3. fetch & insert prices (using the previous 2 datatables)
    // 3a. fetch symbols
    info!("fetching symbols ...");
    let rows = pg_client
        .query("SELECT pk, symbol FROM crypto.symbols", &[])
        .await
        .map_err(|err| {
            error!("failed to fetch crypto.symbols");
            err
        })?;
    let mut symbol_pks: HashMap<String, i32> = HashMap::new();
    for row in rows {
        let pk: i32 = row.get("pk");
        let symbol: String = row.get("symbol");
        symbol_pks.insert(symbol, pk);
    }
    let symbol_pks = Arc::new(symbol_pks);

    // 3b. fetch sources
    info!("fetching sources ...");
    let rows = pg_client
        .query("SELECT pk, source FROM crypto.sources", &[])
        .await
        .map_err(|err| {
            error!("failed to fetch crypto.sources");
            err
        })?;
    let mut source_pks: HashMap<String, i16> = HashMap::new();
    for row in rows {
        let pk: i16 = row.get("pk");
        let source: String = row.get("source");
        source_pks.insert(source, pk);
    }
    let source_pk = match source_pks.get("Binance") {
        Some(pk) => *pk,
        None => {
            error!("failed to find Binance source pk");
            return Err(anyhow::anyhow!("failed to find Binance source pk"));
        }
    };

    use tokio::sync::Mutex;
    let pg_client = Arc::new(Mutex::new(pg_client));

    // 3c. fetch prices for tickers
    info!("fetching prices ...");
    let stream = stream::iter(&tickers.0);
    stream
        .for_each_concurrent(12, |ticker| {
            let http_client = &http_client;
            let pg_client = pg_client.clone();
            let symbol_pks = &symbol_pks;
            let symbol = &ticker.symbol;
            let source_pk = &source_pk;
            async move {
                if let Some(symbol_pk) = symbol_pks.get(symbol) {
                    trace!("fetching prices for {}", symbol);
                    let url = format!(
                    "https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1d&limit=1000"
                );
                    let response = match http_client.get(url).send().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!(
                                "failed to fetch Binance prices for {}, error({err})",
                                &ticker.symbol
                            );
                            return;
                        }
                    };

                    trace!("deserializing prices for {}", symbol);
                    let klines: Klines = match response.json().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("failed to parse Binance prices for {symbol}, error({err})");
                            return;
                        }
                    };

                    let mut pg_client = pg_client.lock().await;
                    match klines
                        .insert(&mut pg_client, symbol, *symbol_pk, *source_pk)
                        .await
                    {
                        Ok(_) => trace!("inserted prices for {symbol}"),
                        Err(err) => error!("failed to insert prices for {symbol}, error({err})"),
                    };
                } else {
                    error!("failed to find symbol pk for {symbol}");
                }
            }
        })
        .await;

    Ok(())
}

// binance http client requires "X-MBX-APIKEY"
fn build_client() -> HttpClient {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-MBX-APIKEY",
        HeaderValue::from_str(&var("BINANCE_API").expect("BINANCE_API not found"))
            .expect("failed to set BINANCE_API as X-MBX-APIKEY header"),
    );
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Binance Client to build");
    client
}

/////////////////////////////////////////////////////////////////////////////////
// endpoints
/////////////////////////////////////////////////////////////////////////////////
//
// tickers
// ----------------------------------------------------------------
// [
//  {
//      "symbol": "ETHBTC",
//      "bidPrice": "0.03699000",
//      "bidQty": "39.03480000",
//      "askPrice": "0.03700000",
//      "askQty": "7.12410000"
//  },
//  {
//      "symbol": "LTCBTC",
//      "bidPrice": "0.00119000",
//      "bidQty": "134.49500000",
//      "askPrice": "0.00119100",
//      "askQty": "58.48100000"
//  },
//  ...
// ]
#[derive(Debug, Deserialize)]
struct Tickers(Vec<Ticker>);

#[derive(Debug, Deserialize)]
struct Ticker {
    symbol: String,
}

impl Tickers {
    async fn insert(&self, pg_client: &mut PgClient) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(sql::INSERT_SYMBOL).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(&self.0);
        while let Some(ticker) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                let result = transaction
                    .execute(&query, &[&ticker.symbol])
                    .await
                    .map_err(|err| {
                        error!("failed to insert symbol data for Binance");
                        err
                    });

                match result {
                    Ok(_) => trace!("inserting Binance symbol data for {}", &ticker.symbol),
                    Err(err) => error!(
                        "failed to insert symbol data for {} from Binance | ERROR: {}",
                        &ticker.symbol, err
                    ),
                };
            }
            .await;
        }

        // unpack the transcation and commit it to the database
        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
            .map_err(|e| {
                error!("failed to commit transaction for symbols from Binance");
                e
            })?;

        debug!(
            "ticker data collected from Binance, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

// prices
// ----------------------------------------------------------------
//
// [
//   [
//     1499040000000,      // Kline open time
//     "0.01634790",       // Open price
//     "0.80000000",       // High price
//     "0.01575800",       // Low price
//     "0.01577100",       // Close price
//     "148976.11427815",  // Volume
//     1499644799999,      // Kline Close time
//     "2434.19055334",    // Quote asset volume
//     308,                // Number of trades
//     "1756.87402397",    // Taker buy base asset volum.e
//     "28.46694368",      // Taker buy quote asset volume
//     "0"                 // Unused field, ignore.
//   ],
//   [
//      ...
//   ],
//   ...
// ]
//
#[derive(Debug, Deserialize)]
struct Klines(Vec<Kline>);

#[derive(Deserialize, Debug)]
struct Kline {
    timestamp: i64,
    opening: String,
    high: String,
    low: String,
    closing: String,
    volume: String,
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
            opening: seq.next_element::<String>()?.expect("String open"),
            high: seq.next_element::<String>()?.expect("String high"),
            low: seq.next_element::<String>()?.expect("String low"),
            closing: seq.next_element::<String>()?.expect("String close"),
            volume: seq.next_element::<String>()?.expect("String volume"),
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
    // insert the vector of Klines to pg rows
    async fn insert(
        self,
        pg_client: &mut PgClient,
        symbol: &String,
        symbol_pk: i32,
        source_pk: i16,
    ) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(sql::INSERT_PRICE).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(self.0);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            let symbol = &symbol;
            let interval_pk: i16 = 3;
            async move {
                let result = transaction
                    .execute(
                        &query,
                        &[
                            &symbol_pk,
                            &chrono::DateTime::from_timestamp_millis(cell.timestamp)
                                .expect("i64 -> DateTime"),
                            &interval_pk,
                            &cell.opening.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.high.parse::<f64>().expect("String -> f64 High"),
                            &cell.low.parse::<f64>().expect("String -> f64 Low"),
                            &cell.closing.parse::<f64>().expect("String -> f64 Closing"),
                            &cell.volume.parse::<f64>().expect("String -> f64 Volume"),
                            &cell.trades,
                            &source_pk,
                        ],
                    )
                    .await;

                match result {
                    Ok(_) => trace!("inserting Binance price data for {symbol}"),
                    Err(err) => {
                        error!("failed to insert price data for {symbol} from Binance, err({err})")
                    }
                }
            }
            .await;
        }

        // unpack the transcation and commit it to the database
        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
            .map_err(|err| {
                error!("failed to commit transaction for {symbol} from Binance, error({err})");
                err
            })?;

        debug!(
            "{symbol} price data collected from Binance, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

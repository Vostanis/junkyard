use super::sql;
use crate::http::*;
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::ClientBuilder;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace};

const BROKERAGE: &str = "Binance";

// RATE_LIMIT = 1200 /60s
//
// tickers = `https://api.binance.com/api/v1/ticker/allBookTickers`
//
// klines = `https://api.binance.com/api/v3/klines`, per symbol

/////////////////////////////////////////////////////////////////////////////////
// core
/////////////////////////////////////////////////////////////////////////////////

pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    // wait for a pg client from the pool
    let mut pg_client = pool.get().await.map_err(|err| {
        error!("failed to get pg client from pool, error({err})");
        err
    })?;

    // fetch the tickers
    if tui {
        println!("{bar}\n{BROKERAGE:^40}\n{bar}", bar = "=".repeat(40))
    }
    let pb = if tui {
        let pb = ProgressBar::new_spinner()
            .with_message("fetching tickers ...")
            .with_style(ProgressStyle::default_spinner().template("{msg} {spinner:.magenta}")?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    } else {
        ProgressBar::hidden()
    };

    
    let http_client = build_client();
    let tickers: Tickers = http_client
        .get("https://api.binance.com/api/v1/ticker/allBookTickers")
        .send()
        .await
        .map_err(|err| {
            error!("failed to fetch {BROKERAGE} tickers, error({err})");
            err
        })?
        .json()
        .await
        .map_err(|err| {
            error!("failed to deserialize {BROKERAGE} tickers, error({err})");
            err
        })?;
    pb.set_message("inserting tickers ...");

    pg_client
        .query(
            "INSERT INTO crypto.sources (source) VALUES ($1) ON CONFLICT DO NOTHING",
            &[&BROKERAGE],
        )
        .await
        .map_err(|err| {
            error!("failed to insert {BROKERAGE} as a source, error({err})");
            err
        })?;

    // 2. insert tickers
    tickers.insert(&mut pg_client).await?;
    pb.finish_with_message("inserting tickers ... done");

    // 3. fetch & insert prices (using the previous 2 datatables)
    // 3a. fetch symbols
    info!("fetching symbols ...");
    let symbol_pks: HashMap<String, i32> = super::util::fetch_pks(
        &mut pg_client,
        "SELECT pk, symbol FROM crypto.symbols",
        "symbol",
        "pk",
    )
    .await
    .map_err(|err| {
        error!("failed to fetch symbols, error({err})");
        err
    })?;
    let symbol_pks = Arc::new(symbol_pks);

    // 3b. fetch sources
    info!("fetching sources ...");
    let source_pks: HashMap<String, i16> = super::util::fetch_pks(
        &mut pg_client,
        "SELECT pk, source FROM crypto.sources",
        "source",
        "pk",
    )
    .await
    .map_err(|err| {
        error!("failed to fetch sources, error({err})");
        err
    })?;
    let source_pk = match source_pks.get(BROKERAGE) {
        Some(pk) => *pk,
        None => {
            error!("failed to find {BROKERAGE} source pk");
            return Err(anyhow::anyhow!("failed to find {BROKERAGE} source pk"));
        }
    };

    drop(pg_client);

    // progress bar
    let (multi, total, success, fail) = if tui {
        crate::tui::multi_progress(tickers.0.len())?
    } else {
        (None, None, None, None)
    };

    // 3c. fetch prices for tickers
    info!("fetching prices ...");
    let stream = stream::iter(&tickers.0);
    stream
        .for_each_concurrent(num_cpus::get(), |ticker| {
            let http_client = &http_client;
            let symbol_pks = &symbol_pks;
            let symbol = &ticker.symbol;
            let source_pk = &source_pk;

            // progress bars
            let multi = multi.clone();
            let total = total.clone();
            let success = success.clone();
            let fail = fail.clone();
            async move {
                if let Some(symbol_pk) = symbol_pks.get(symbol) {
                    trace!("fetching prices for {}", symbol);

                    // if tui is enabled, create a progress bar, per task currently being executed
                    let spinner = crate::tui::multi_progress_spinner(multi, format!("fetching prices for {symbol}"));
                    spinner.enable_steady_tick(Duration::from_millis(50));

                    let url = format!(
                        "https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1d&limit=1000"
                    );


                    let response = match http_client.get(url).send().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!(
                                "failed to fetch {BROKERAGE} prices for {symbol}, error({err})",
                            );

                            if tui {
                                fail.expect("failbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }

                            return;
                        }
                    };

                    trace!("deserializing prices for {}", symbol);
                    spinner.set_message(format!("deserializing prices for {symbol}"));
                    let klines: Klines = match response.json().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("failed to parse {BROKERAGE} prices for {symbol}, error({err})");

                            if tui {
                                fail.expect("failbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }

                            return;
                        }
                    };

                    spinner.set_message(format!("waiting to insert prices for {symbol}"));
                    let mut pg_client = match pool.get().await {
                        Ok(client) => client,
                        Err(err) => {
                            error!("failed to get pg client from pool, error({err})");
                            return;
                        }
                    };

                    spinner.set_message(format!("inserting prices for {symbol}"));
                    match klines
                        .insert(&mut pg_client, symbol, *symbol_pk, *source_pk)
                        .await
                    {
                        Ok(_) => {
                            trace!("inserted prices for {symbol}");
                            
                            if tui {
                                success.expect("successbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }
                        },
                        Err(err) => {
                            error!("failed to insert prices for {symbol}, error({err})");

                            if tui {
                                fail.expect("failbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }
                        },
                    };
                } else {
                    error!("failed to find symbol pk for {symbol}");

                    if tui {
                        fail.expect("failbar should have unwrapped").inc(1);
                        total.expect("totalbar should have unwrapped").inc(1);
                    }
                }
            }
        })
        .await;

    fail.expect("fail bar should have unwrapped")
        .finish_and_clear();
    success
        .expect("success bar should have unwrapped")
        .finish_and_clear();
    total
        .expect("total bar should have unwrapped")
        .finish_and_clear();

    if tui {
        println!("collecting crypto prices ... done\n");
    }

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
        .expect("Binance client to build");
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
                        error!("failed to insert symbol data for {BROKERAGE}, error({err})");
                        err
                    });

                match result {
                    Ok(_) => trace!("inserting {BROKERAGE} symbol data for {}", &ticker.symbol),
                    Err(err) => error!(
                        "failed to insert symbol data for {} from {BROKERAGE}, error({})",
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
            .map_err(|err| {
                error!("failed to commit transaction for symbols from {BROKERAGE}");
                err
            })?;

        debug!(
            "ticker data collected from {BROKERAGE}, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
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
                    Ok(_) => trace!("inserting {BROKERAGE} price data for {symbol}"),
                    Err(err) => {
                        error!(
                            "failed to insert price data for {symbol} from {BROKERAGE}, err({err})"
                        )
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
                error!("failed to commit transaction for {symbol} from {BROKERAGE}, error({err})");
                err
            })?;

        debug!(
            "{symbol} price data collected from {BROKERAGE}, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

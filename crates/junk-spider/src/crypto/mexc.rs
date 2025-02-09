use super::sql;
use crate::http::*;
use deadpool_postgres::Pool;
use dotenv::var;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::HeaderValue;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace};

const BROKERAGE: &'static str = "MEXC";

// RATE_LIMIT = 500 /1s
//
// tickers = `https://api.mexc.com/api/v3/ticker/bookTicker`
//
// klines = `https://api.mexc.com/api/v3/klines?symbol=BTCUSDT&interval=1d`, per symbol

/////////////////////////////////////////////////////////////////////////////////
// core
/////////////////////////////////////////////////////////////////////////////////

pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    // wait for a pg client from the pool
    let mut pg_client = pool.get().await.map_err(|err| {
        error!("failed to get pg client from the pool, error({err})");
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
        .get("https://api.mexc.com/api/v3/ticker/bookTicker")
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

    // 1. insert source
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

    // 2. insert tickers to pg rows
    pb.set_message("inserting tickers ...");
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
    .await?;
    let symbol_pks = Arc::new(symbol_pks);

    // 3b. fetch sources
    info!("fetching sources ...");
    let source_pks: HashMap<String, i16> = super::util::fetch_pks(
        &mut pg_client,
        "SELECT pk, source FROM crypto.sources",
        "source",
        "pk",
    )
    .await?;
    let source_pk = match source_pks.get(BROKERAGE) {
        Some(pk) => *pk,
        None => {
            error!("failed to find {BROKERAGE} source pk");
            return Err(anyhow::anyhow!("failed to find {BROKERAGE} source pk"));
        }
    };

    drop(pg_client);

    // progress bars
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

            // progress bar
            let multi = multi.clone();
            let total = total.clone();
            let success = success.clone();
            let fail = fail.clone();
            async move {
                if let Some(symbol_pk) = symbol_pks.get(symbol) {
                    trace!("fetching prices for {}", symbol);

                    // if tui is enabled, create a progress bar, per task currently being executed
                    let spinner = multi.unwrap_or_default().add(
                        ProgressBar::new_spinner()
                            .with_message(format!("fetching prices for {symbol}"))
                            .with_style(
                                ProgressStyle::default_spinner()
                                    .template("\t   > {msg}")
                                    .expect("failed to set spinner style"),
                            ),
                    );
                    spinner.enable_steady_tick(Duration::from_millis(50));

                    let url = format!(
                        "https://api.mexc.com/api/v3/klines?symbol={symbol}&interval=1d&limit=1000"
                    );
                    let response = match http_client.get(url).send().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("failed to fetch {BROKERAGE} prices for {symbol}, error({err})");

                            if tui {
                                fail.expect("failbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }


                            return;
                        }
                    };

                    let klines: Klines = match response.json().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!("failed to deserialize {BROKERAGE} prices for {symbol}, error({err})");
                            if tui {
                                fail.expect("failbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }


                            return;
                        }
                    };

                    // wait for a client
                    spinner.set_message(format!("waiting to insert prices for {symbol}"));
                    let mut pg_client = match pool.get().await {
                        Ok(client) => client,
                        Err(err) => {
                            error!("failed to get pg client from the pool, error({err})");

                            if tui {
                                fail.expect("failbar should have unwrapped").inc(1);
                                total.expect("totalbar should have unwrapped").inc(1);
                            }

                            return;
                        }
                    };

                    match klines
                        .insert(&mut pg_client, symbol.to_string(), *symbol_pk, *source_pk)
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
    println!("collecting crypto prices ... done\n");

    Ok(())
}

fn build_client() -> HttpClient {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "apiKey",
        HeaderValue::from_str(&var("MEXC_API").expect("MEXC_API not found"))
            .expect("failed to set MEXC_API as `apiKey` header"),
    );
    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("MEXC client to build");
    client
}

/////////////////////////////////////////////////////////////////////////////////
// endpoints
/////////////////////////////////////////////////////////////////////////////////
//
// NOTE: All elements of the array are Strings
//
// tickers
// ----------------------------------------------------------------
// {
//      "code": "200000",
//      "data": {
//          "time": 1734700773024,
//          "ticker": [
//              {
//                  "symbol": "HLG-USDT",
//                  "symbolName": "HLG-USDT",
//                  "buy": "0.00127",
//                  "bestBidSize": "117388.1",
//                  "sell": "0.0013",
//                  "bestAskSize": "4581.9",
//                  "changeRate": "-0.0921",
//                  "changePrice": "-0.00013",
//                  "high": "0.00143",
//                  "low": "0.00121",
//                  "vol": "11114534.6",
//                  "volValue": "14723.709605",
//                  "last": "0.00128",
//                  "averagePrice": "0.00161573",
//                  "takerFeeRate": "0.001",
//                  "makerFeeRate": "0.001",
//                  "takerCoefficient": "2",
//                  "makerCoefficient":
//              },
//              ...
//          ]
//      },
// }
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
//[
//  [
//      1640804880000,
//      "47482.36",
//      "47482.36",
//      "47416.57",
//      "47436.1",
//      "3.550717",
//      1640804940000,
//      "168387.3"
//  ],
//  [
//      ...
//  ],
//  ...
// ]
#[derive(Deserialize, Debug)]
pub struct Klines(Vec<Kline>);

#[derive(Deserialize, Debug)]
struct Kline {
    timestamp: i64,
    opening: String,
    closing: String,
    high: String,
    low: String,
    volume: String,
    _close_time: IgnoredAny,
    _turnover: IgnoredAny,
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
            timestamp: seq.next_element()?.expect("number timestamp"),
            opening: seq.next_element()?.expect("String opening"),
            closing: seq.next_element()?.expect("String closing"),
            high: seq.next_element()?.expect("String high"),
            low: seq.next_element()?.expect("String low"),
            volume: seq.next_element()?.expect("String volume"),
            _close_time: seq
                .next_element::<IgnoredAny>()?
                .expect("number close_time"),
            _turnover: seq.next_element::<IgnoredAny>()?.expect("String turnover"),
        })
    }
}

impl Klines {
    // insert the vector of Klines to pg rows
    async fn insert(
        self,
        pg_client: &mut PgClient,
        symbol: String,
        symbol_pk: i32,
        source_pk: i16,
    ) -> anyhow::Result<()> {
        let time = ::std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&sql::INSERT_PRICE).await?;
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
                            &chrono::DateTime::from_timestamp_millis(cell.timestamp),
                            &interval_pk,
                            &cell.opening.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.high.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.low.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.closing.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.volume.parse::<f64>().expect("String -> f64 Opening"),
                            &None::<i64>,
                            &source_pk,
                        ],
                    )
                    .await;

                match result {
                    Ok(_) => trace!("inserting {BROKERAGE} price data for {symbol}"),
                    Err(err) => {
                        error!("failed to insert price data for {symbol} from {BROKERAGE}, error({err})")
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
                error!("failed to commit transaction for {symbol} from {BROKERAGE}");
                err
            })?;

        debug!(
            "{symbol} price data collected from {BROKERAGE}, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

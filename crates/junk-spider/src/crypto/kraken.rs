use super::sql;
use crate::http::*;
use dotenv::var;
use futures::{stream, StreamExt};
use reqwest::header::HeaderValue;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

const BROKERAGE: &'static str = "Kraken";

// RATE_LIMIT = 15 /1s
//
// tickers = `https://api.kucoin.com/api/v1/market/allTickers`
//
// NOTE: KuCoin symbols include a dash, e.g. BTC-USDT, or ETH-BTC
//
// klines = `https://api.kucoin.com/api/v1/market/candles?type=1day&symbol=BTC-USDT`, per symbol

/////////////////////////////////////////////////////////////////////////////////
// core
/////////////////////////////////////////////////////////////////////////////////

pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
    let http_client = build_client();

    // fetch the tickers
    let tickers: KrakenSymbols = http_client
        .get("https://api.kraken.com/0/public/AssetPairs")
        .send()
        .await
        .map_err(|err| {
            error!("failed to fetch {BROKERAGE} tickers, error({err})");
            err
        })?
        .json()
        .await
        .map_err(|err| {
            error!("failed to dserialize {BROKERAGE} tickers, error({err})");
            err
        })?;

    // 1. insert kucoin source
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
    tickers.insert(pg_client).await?;

    // 3. fetch & insert prices (using the previous 2 datatables)
    // 3a. fetch symbols
    info!("fetching symbols ...");
    let rows = pg_client
        .query("SELECT pk, symbol FROM crypto.symbols", &[])
        .await
        .map_err(|err| {
            error!("failed to fetch crypto.symbols, error({err})");
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
            error!("failed to fetch crypto.sources, error({err})");
            err
        })?;
    let mut source_pks: HashMap<String, i16> = HashMap::new();
    for row in rows {
        let pk: i16 = row.get("pk");
        let source: String = row.get("source");
        source_pks.insert(source, pk);
    }
    let source_pk = match source_pks.get(BROKERAGE) {
        Some(pk) => *pk,
        None => {
            error!("failed to find {BROKERAGE} source pk");
            return Err(anyhow::anyhow!("failed to find {BROKERAGE} source pk"));
        }
    };

    let pg_client = Arc::new(Mutex::new(pg_client));

    // 3c. fetch prices for tickers
    info!("fetching prices ...");
    let stream = stream::iter(&tickers.result);
    stream
        .for_each_concurrent(12, |(_key, value)| {
            let http_client = &http_client;
            let pg_client = pg_client.clone();
            let symbol_pks = &symbol_pks;
            let symbol = &value.altname;
            let source_pk = &source_pk;

            async move {
                if let Some(symbol_pk) = symbol_pks.get(symbol) {
                    trace!("fetching prices for {}", symbol);
                    let url = format!(
                        // NOTE: intervals are in minute intervals
                        // 1, 5, 15, 30, 60, 240, 1440, 10080, 21600
                        "https://api.kraken.com/0/public/OHLC?interval=1440&pair={symbol}"
                    );
                    let response = match http_client
                        .get(url)
                        .send()
                        .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            error!("failed to fetch {BROKERAGE} prices for {symbol}, error({err})");
                            return;
                        }
                    };

                    let klines: Klines = match response.json().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!(
                                "failed to deserialize {BROKERAGE} prices for {symbol}, error({err})"
                            );
                            return;
                        }
                    };

                    let mut pg_client = pg_client.lock().await;
                    match klines
                        .insert(&mut pg_client, symbol.to_string(), *symbol_pk, *source_pk)
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

fn build_client() -> HttpClient {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "API-Key",
        HeaderValue::from_str(&var("KRAKEN_API").expect("KRAKEN_API not found"))
            .expect("failed to set KRAKEN_API as API-Key header"),
    );
    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("KRAKEN client to build");
    client
}

// // security
// // ----------------------------------------------------------------
// //
// // https://docs.kraken.com/api/docs/guides/spot-rest-auth
//
// fn encrypt(secret: String, input: String) -> String {
//     type HmacSha256 = Hmac<Sha256>;
//     let mut mac = HmacSha256::new_from_slice(&secret.as_bytes()).unwrap();
//     mac.update(input.as_bytes());
//     let result = mac.finalize().into_bytes();
//     let b64 = BASE64_STANDARD.encode(&result);
//     b64
// }
//
// fn sign(url: &String, secret: String, nonce: &String) -> String {
//     let url = url.replace("https://api.kraken.com", "");
//     let input = format!("{}{}{}", nonce, "GET", url);
//     encrypt(secret, input)
// }
//
// fn nonce() -> String {
//     chrono::Utc::now().timestamp_millis().to_string()
// }

/////////////////////////////////////////////////////////////////////////////////
// endpoints
/////////////////////////////////////////////////////////////////////////////////
//
// NOTE: All elements of the array are Strings
//
// tickers
// ----------------------------------------------------------------
//
//  {
//      "error": [],
//      "result": {
//          "XETHXXBT": {
//              "altname": "ETHXBT",
//              "wsname": "ETH/XBT",
//              "aclass_base": "currency",
#[derive(Debug, Deserialize)]
struct KrakenSymbols {
    // error: Vec<String>,
    result: HashMap<String, Pair>,
}

#[derive(Debug, Deserialize)]
struct Pair {
    altname: String,
}

impl KrakenSymbols {
    async fn insert(&self, pg_client: &mut PgClient) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(sql::INSERT_SYMBOL).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(&self.result);
        while let Some((_key, value)) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                let result = transaction
                    .execute(&query, &[&value.altname])
                    .await
                    .map_err(|err| {
                        error!("failed to insert symbol data for {BROKERAGE}, error({err})");
                        err
                    });

                match result {
                    Ok(_) => trace!("inserting {BROKERAGE} symbol data for {}", &value.altname),
                    Err(err) => error!(
                        "failed to insert symbol data for {} from {BROKERAGE}, error({err})",
                        &value.altname
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
                error!("failed to commit transaction for symbols from {BROKERAGE}, error({err})");
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
//  {
//      "error": [],
//      "result": {
//          "XXBTZUSD": [
//              [
//                  1688671200,
//                  "30306.1",
//                  "30306.2",
//                  "30305.7",
//                  "30305.7",
//                  "30306.1",
//                  "3.39243896",
//                  23
//              ],
//              ...
//          ],
//      "last": 1678234233,
//  }
#[derive(Deserialize, Debug)]
pub struct Klines {
    result: ResultData,
}

#[derive(Deserialize, Debug)]
struct ResultData {
    #[serde(flatten)]
    pairs: HashMap<String, Vec<Kline>>,

    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    last: u64,
}

#[derive(Deserialize, Debug)]
struct Kline {
    time: i64,
    opening: String,
    closing: String,
    high: String,
    low: String,
    _vwap: IgnoredAny,
    volume: String,
    trades: i32,
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
            time: seq.next_element()?.expect("i64 timestamp"),
            opening: seq.next_element()?.expect("String opening"),
            closing: seq.next_element()?.expect("String closing"),
            high: seq.next_element()?.expect("String high"),
            low: seq.next_element()?.expect("String low"),
            _vwap: seq.next_element::<IgnoredAny>()?.expect("String volume"),
            volume: seq.next_element()?.expect("String volume"),
            trades: seq.next_element()?.expect("i32 trades"),
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

        // iterate over the data stream and execute pg rows
        for (_result_symbol, klines) in self.result.pairs {
            // preprocess pg query as transaction
            let query = pg_client.prepare(&sql::INSERT_PRICE).await?;
            let transaction = Arc::new(pg_client.transaction().await?);

            let mut stream = stream::iter(klines);
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
                                &chrono::DateTime::from_timestamp(
                                    cell.time,
                                    0,
                                ),
                                &interval_pk,
                                &cell.opening.parse::<f64>().expect("String -> f64 Opening"),
                                &cell.high.parse::<f64>().expect("String -> f64 Opening"),
                                &cell.low.parse::<f64>().expect("String -> f64 Opening"),
                                &cell.closing.parse::<f64>().expect("String -> f64 Opening"),
                                &cell.volume.parse::<f64>().expect("String -> f64 Opening"),
                                &cell.trades,
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
                    error!(
                        "failed to commit transaction for {symbol} from {BROKERAGE}, error({err})"
                    );
                    err
                })?;

            debug!(
                "{symbol} price data collected from {BROKERAGE}, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
                time.elapsed().as_millis()
            );
        }

        Ok(())
    }
}

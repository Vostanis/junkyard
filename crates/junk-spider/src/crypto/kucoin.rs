use super::sql;
use crate::http::*;
use base64::prelude::{Engine, BASE64_STANDARD};
use dotenv::var;
use futures::{stream, StreamExt};
use hmac::{Hmac, Mac};
use reqwest::header::HeaderValue;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

// RATE_LIMIT = 4000 /30s
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
    let tickers: KuCoinTickerResponse = http_client
        .get("https://api.kucoin.com/api/v1/market/allTickers")
        .send()
        .await
        .map_err(|err| {
            error!("failed to fetch KuCoin tickers");
            err
        })?
        .json()
        .await
        .map_err(|err| {
            error!("failed to dserialize KuCoin tickers");
            err
        })?;

    // 1. insert kucoin source
    pg_client
        .query(
            "INSERT INTO crypto.sources (source) VALUES ('KuCoin') ON CONFLICT DO NOTHING",
            &[],
        )
        .await
        .map_err(|err| {
            error!("failed to insert KuCoin as a source");
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
    let source_pk = match source_pks.get("KuCoin") {
        Some(pk) => *pk,
        None => {
            error!("failed to find KuCoin source pk");
            return Err(anyhow::anyhow!("failed to find KuCoin source pk"));
        }
    };

    let pg_client = Arc::new(Mutex::new(pg_client));

    // 3c. fetch prices for tickers
    info!("fetching prices ...");
    let stream = stream::iter(&tickers.data.ticker);
    stream
        .for_each_concurrent(12, |ticker| {
            let http_client = &http_client;
            let pg_client = pg_client.clone();
            let symbol_pks = &symbol_pks;
            let symbol = &ticker.symbol;
            let source_pk = &source_pk;
            let private = var("KUCOIN_PRIVATE").expect("KUCOIN_PRIVATE not found");
            let passphrase = var("KUCOIN_PASSPHRASE").expect("KUCOIN_PASSPHRASE not found");

            async move {
                if let Some(symbol_pk) = symbol_pks.get(&symbol.replace("-", "")) {
                    trace!("fetching prices for {}", symbol);
                    let url = format!(
                        "https://api.kucoin.com/api/v1/market/candles?type=1day&symbol={symbol}"
                    );
                    let timestamp = timestamp();
                    let passphrase = encrypt(private.clone(), passphrase);
                    let sign = sign(&url, private, timestamp.clone());
                    let response = match http_client
                        .get(url)
                        .header("KC-API-TIMESTAMP", timestamp)
                        .header("KC-API-PASSPHRASE", passphrase)
                        .header("KC-API-SIGN", sign)
                        .send()
                        .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            error!("failed to fetch KuCoin prices for {symbol}, error({err})");
                            return;
                        }
                    };

                    let klines: Klines = match response.json().await {
                        Ok(data) => data,
                        Err(err) => {
                            error!(
                                "failed to deserialize KuCoin prices for {symbol}, error({err})"
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
        "KC-API-KEY",
        HeaderValue::from_str(&var("KUCOIN_API").expect("KUCOIN_API not found"))
            .expect("failed to set KUCOIN_API as X-MBX-APIKEY header"),
    );
    headers.insert(
        "KC-API-VERSION",
        HeaderValue::from_str(&"2").expect("failed to set kc-api-version to \"2\""),
    );
    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("KuCoin Client to build");
    client
}

// security
// ----------------------------------------------------------------
//
// https://www.kucoin.com/docs/basic-info/connection-method/authentication/signing-a-message

fn encrypt(secret: String, input: String) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&secret.as_bytes()).unwrap();
    mac.update(input.as_bytes());
    let result = mac.finalize().into_bytes();
    let b64 = BASE64_STANDARD.encode(&result);
    b64
}

fn sign(url: &String, secret: String, timestamp: String) -> String {
    let url = url.replace("https://api.kucoin.com", "");
    let input = format!("{}{}{}", timestamp, "GET", url);
    encrypt(secret, input)
}

fn timestamp() -> String {
    chrono::Utc::now().timestamp_millis().to_string()
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
struct KuCoinTickerResponse {
    // code: String,
    data: KuCoinData,
}

#[derive(Debug, Deserialize)]
struct KuCoinData {
    ticker: Vec<Ticker>,
}

#[derive(Debug, Deserialize)]
struct Ticker {
    symbol: String,
}

impl KuCoinTickerResponse {
    async fn insert(&self, pg_client: &mut PgClient) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(sql::INSERT_SYMBOL).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(&self.data.ticker);
        while let Some(ticker) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                let result = transaction
                    .execute(&query, &[&ticker.symbol.replace("-", "")])
                    .await
                    .map_err(|err| {
                        error!("failed to insert symbol data for KuCoin");
                        err
                    });

                match result {
                    Ok(_) => trace!("inserting KuCoin symbol data for {}", &ticker.symbol),
                    Err(err) => error!(
                        "failed to insert symbol data for {} from KuCoin, error({})",
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
                error!("failed to commit transaction for symbols from KuCoin");
                e
            })?;

        debug!(
            "ticker data collected from KuCoin, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
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
//      time 	        // Start time of the candle cycle
//      open 	        // Opening price
//      close 	        // Closing price
//      high 	        // Highest price
//      low 	        // Lowest price
//      volume 	        // Transaction volume(One-sided transaction volume)
//      turnover 	// Transaction amount(One-sided transaction amount)
//  ],
//  [
//      ...
//  ],
//  ...
// ]
#[derive(Deserialize, Debug)]
pub struct Klines {
    data: Vec<Kline>,
}

#[derive(Deserialize, Debug)]
struct Kline {
    time: String,
    opening: String,
    closing: String,
    high: String,
    low: String,
    volume: String,
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
            time: seq.next_element()?.expect("String timestamp"),
            opening: seq.next_element()?.expect("String opening"),
            closing: seq.next_element()?.expect("String closing"),
            high: seq.next_element()?.expect("String high"),
            low: seq.next_element()?.expect("String low"),
            volume: seq.next_element()?.expect("String volume"),
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
        let mut stream = stream::iter(self.data);
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
                                cell.time.parse::<i64>().expect("String -> i64 Time"),
                                0,
                            ),
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
                    Ok(_) => trace!("inserting KuCoin price data for {symbol}"),
                    Err(err) => {
                        error!("failed to insert price data for {symbol} from KuCoin, error({err})")
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
                error!("failed to commit transaction for {symbol} from KuCoin");
                err
            })?;

        debug!(
            "{symbol} price data collected from KuCoin, \x1b[38;5;208melapsed time: {} ms\x1b[0m",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

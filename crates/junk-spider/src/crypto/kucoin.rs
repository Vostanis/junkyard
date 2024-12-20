use super::sql;
use crate::http::*;
use base64::prelude::{Engine, BASE64_STANDARD};
use dotenv::var;
use futures::{stream, StreamExt};
use hmac::{Hmac, Mac};
use reqwest::header::HeaderValue;
use serde::de::{SeqAccess, Visitor};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;
use tracing::{debug, error, trace};

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
            error!("failed to fetch Binance tickers");
            err
        })?
        .json()
        .await
        .map_err(|err| {
            error!("failed to dserialize Binance tickers");
            err
        })?;

    println!("{:#?}", tickers.data.ticker);

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
    #[serde(deserialize_with = "de_symbol")]
    symbol: String,
}

/// Remove any dashes from KuCoin symbols, e.g. "BTC-USDT" -> "BTCUSDT"
fn de_symbol<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    Ok(value.replace("-", ""))
}

impl KuCoinTickerResponse {
    async fn insert(&self, pg_client: &mut PgClient) -> anyhow::Result<()> {}
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
pub struct Klines(Vec<Kline>);

#[derive(Deserialize, Debug)]
struct Kline {
    time: String,
    opening: String,
    closing: String,
    high: String,
    low: String,
    volume: String,
    turnover: String,
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
            turnover: seq.next_element()?.expect("String turnover"),
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
                            &cell
                                .turnover
                                .parse::<f64>()
                                .expect("String -> f64 Turnover"),
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

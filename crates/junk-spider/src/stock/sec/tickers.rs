use crate::http::*;
use crate::stock::common::de_cik;
use futures::{stream, StreamExt};
use serde::de::Visitor;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, trace};

// scrape
// ----------------------------------------------------------------------------

async fn scrape() -> anyhow::Result<()> {
    let client = build_client();
    let response = client
        .get("https://www.sec.gov/files/company_tickers.json")
        .send()
        .await?;
    let body = response.json().await?;
    Ok(())
}

fn build_client() -> HttpClient {
    reqwest::ClientBuilder::new()
        .user_agent(var("USER_AGENT").expect("failed to read USER_AGENT"))
        .build()
        .expect("failed to build reqwest client")
}

// de
// ----------------------------------------------------------------------------

#[derive(Debug)]
struct Tickers(Vec<Ticker>);

// Individual stock behaviour; i.e., each ticker in the list needs to process price & metrics
// data (and any tertiary data) separately.
#[derive(Clone, Debug, Deserialize)]
struct Ticker {
    #[serde(rename = "cik_str", deserialize_with = "de_cik")]
    cik: String,
    ticker: String,
    title: String,
}

struct TickerVisitor;

impl<'de> Visitor<'de> for TickerVisitor {
    type Value = Tickers;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Map of tickers")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // each entry is in the form of:
        // `0: { "cik_str": 320193, "ticker": "AAPL", "title": "Apple Inc." },
        //  1: { ... },
        //  ...`
        let mut tickers: Vec<Ticker> = Vec::new();
        while let Some((_, ticker)) = map.next_entry::<u16, Ticker>().expect("next_entry") {
            tickers.push(ticker);
        }
        Ok(Tickers(tickers))
    }
}

impl<'de> Deserialize<'de> for Tickers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // we want a vector returned, but the deserialize will expect a map, given
        // how the API has been designed
        deserializer.deserialize_map(TickerVisitor)
    }
}

// pg
// ----------------------------------------------------------------------------

impl Tickers {
    async fn insert(pg_client: &mut PgClient) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&INDEX_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(data.0);
        while let Some(cell) = stream.next().await {
            let path = format!("./buffer/submissions/CIK{}.json", cell.stock_id);
            trace!("reading file at path: \"{path}\"");
            let file: Sic = match read_json(&path).await {
                Ok(data) => data,
                Err(e) => {
                    error!("failed to read file | {e}");
                    continue;
                }
            };

            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                match transaction
                    .execute(
                        &query,
                        &[
                            &cell.stock_id,
                            &cell.ticker,
                            &cell.title,
                            &file.sic_description,
                            &"US",
                        ],
                    )
                    .await
                {
                    Ok(_) => trace!("Stock index inserted"),
                    Err(err) => error!("Failed to insert SEC Company Tickers | ERROR: {err}"),
                }
            }
            .await;
        }

        // unpack the transcation and commit it to the database
        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
            .map_err(|e| {
                error!("failed to commit transaction for SEC Company Tickers");
                e
            })?;

        debug!(
            "Binance priceset inserted. Elapsed time: {} ms",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

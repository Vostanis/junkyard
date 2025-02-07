use crate::stock::common::de_cik;
use crate::{http::*, stock::sql};
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::de::Visitor;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, trace};

pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    let client = build_client();

    debug!("fetching SEC Company Tickers");
    if tui {
        println!(
            "{bar}\n{name:^40}\n{bar}",
            bar = "=".repeat(40),
            name = "SEC Tickers"
        );
    }
    let tickers: Tickers = client
        .get("https://www.sec.gov/files/company_tickers.json")
        .send()
        .await
        .map_err(|err| {
            error!("failed to fetch data, error({err})");
            err
        })?
        .json()
        .await
        .map_err(|err| {
            error!("failed to parse JSON, error({err})");
            err
        })?;

    let pg_client = &mut pool.get().await?;
    tickers.insert(pg_client, tui).await?;

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
pub struct Tickers(Vec<Ticker>);

// Individual stock behaviour; i.e., each ticker in the list needs to process price & metrics
// data (and any tertiary data) separately.
#[derive(Clone, Debug, Deserialize)]
pub struct Ticker {
    #[serde(rename = "cik_str", deserialize_with = "de_cik")]
    pub pk: String,
    pub ticker: String,
    pub title: String,
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

impl Tickers {
    async fn insert(&self, pg_client: &mut PgClient, tui: bool) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&sql::INSERT_TICKER).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // progress bar
        let pb = if tui {
            let pb = ProgressBar::new(self.0.len() as u64).with_style(
                ProgressStyle::default_bar()
                    .template(
                        "{msg} {spinner:.magenta}\n\
                        [{elapsed_precise:.magenta}] |{bar:40.cyan/blue}| {human_pos}/{human_len} \
                        [Rate: {per_sec:.magenta}, ETA: {eta:.blue}]",
                    )?
                    .progress_chars("##-"),
            );
            pb.set_message("collecting tickers ...");
            pb.enable_steady_tick(Duration::from_millis(100));
            pb
        } else {
            ProgressBar::hidden()
        };

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(&self.0);
        while let Some(cell) = stream.next().await {
            let path = format!("./buffer/submissions/CIK{}.json", cell.pk);
            let file: Sic = match crate::fs::read_json(&path).await {
                Ok(data) => data,
                Err(err) => {
                    error!("failed to read file, error({err})");
                    continue;
                }
            };

            let query = query.clone();
            let transaction = transaction.clone();

            let pb = pb.clone();
            async move {
                match transaction
                    .execute(
                        &query,
                        &[
                            &cell.pk,
                            &cell.ticker,
                            &cell.title.to_uppercase(),
                            &file.sic_description,
                            &"US",
                        ],
                    )
                    .await
                {
                    Ok(_) => {
                        trace!("stock tickers inserted");
                        pb.inc(1)
                    }
                    Err(err) => error!("failed to insert SEC Company Tickers, error({err})"),
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
                error!("failed to commit transaction for SEC Company Tickers");
                err
            })?;

        debug!("SEC stock tickers inserted. {}", crate::time_elapsed(time));

        pb.finish_and_clear();

        if tui {
            println!("collecting tickers ... done\n");
        }

        Ok(())
    }
}

// Struct for the SIC code retrived from submission files.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sic {
    sic_description: String,
}

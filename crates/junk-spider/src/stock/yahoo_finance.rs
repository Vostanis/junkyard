use super::sql;
use crate::http::*;
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace};

// scrape
// ----------------------------------------------------------------------------

pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    // wait for a pg client from the pool
    let pg_client = pool.get().await.map_err(|err| {
        error!("failed to get pg client from pool, error({err})");
        err
    })?;

    // return all tickers from the database
    if tui {
        println!(
            "{bar}\n{source:^40}\n{bar}",
            bar = "=".repeat(40),
            source = "Yahoo! Finance"
        )
    }
    info!("fetching stock.tickers ...");
    let tickers: Vec<Ticker> = pg_client
        .query("SELECT pk, symbol, title FROM stock.symbols", &[])
        .await
        .map_err(|err| {
            error!("failed to fetch stock.symbols, error({err})");
            err
        })?
        .into_par_iter()
        .map(|row| Ticker {
            pk: row.get(0),
            ticker: row.get(1),
            title: row.get(2),
        })
        .collect();

    drop(pg_client);

    // progress bar
    let num = tickers.len();
    let (multi, total, success, fail) = if tui {
        // overall multi progress bar
        let multi = MultiProgress::new();

        // total number of tickers to collect
        let total = multi.add(
            ProgressBar::new(num as u64).with_style(
                ProgressStyle::default_bar()
                    .template("collecting stock prices ... {spinner:.magenta}\n \
                        {msg:>9.white} |{bar:57.white/grey}| {pos:<2} / {human_len} ({percent_precise}%)\
                        [Time: {elapsed}, Rate: {per_sec}, ETA: {eta}]")?
                    .progress_chars("## "),
            ),
        );
        total.set_message("total");
        total.enable_steady_tick(Duration::from_millis(100));

        // total successful collections
        let success = multi.insert_after(
            &total,
            ProgressBar::new(num as u64).with_style(
                ProgressStyle::default_bar()
                    .template(" {msg:>9.green} |{bar:57.green}| {pos:<2.green}")?
                    .progress_chars("## "),
            ),
        );
        success.set_message("successes");

        // total failed collections
        let fails = multi.insert_after(
            &success,
            ProgressBar::new(num as u64).with_style(
                ProgressStyle::default_bar()
                    .template(" {msg:>9.red} |{bar:57.red}| {pos:<2.red}")?
                    .progress_chars("## "),
            ),
        );
        fails.set_message("failures");

        (Some(multi), Some(total), Some(success), Some(fails))
    } else {
        (None, None, None, None)
    };

    // stream over tickers and fetch prices from Yahoo Finance
    info!("fetching Yahoo Finance prices ...");
    let http_client = crate::std_client_build();
    let stream = stream::iter(tickers);
    stream
        .for_each_concurrent(num_cpus::get(), |ticker| {
        let http_client = &http_client;

        // progress bars
        let multi = multi.clone();
        let total = total.clone();
        let success = success.clone();
        let fail = fail.clone();
        async move {
            // if tui is enabled, create a progress bar, per task currently being executed
            let spinner = multi.unwrap_or_default().add(
                ProgressBar::new_spinner()
                    .with_message(format!("fetching [{}] {}", &ticker.ticker, &ticker.title))
                    .with_style(
                        ProgressStyle::default_spinner()
                            .template("\t   > {msg}")
                            .expect("failed to set spinner style"),
                    ),
            );
            spinner.enable_steady_tick(Duration::from_millis(50));

            let url = format!(
                "https://query2.finance.yahoo.com/v8/finance/chart/{}?range=10y&interval=1d",
                &ticker.ticker
            );

            // fetch raw http response
            let response = match http_client.get(url).send().await {
                Ok(response) => response,
                Err(err) => {
                    error!(
                        "failed to fetch Yahoo Finance prices for [{}] {}, error({err})",
                        &ticker.ticker, &ticker.title
                    );

                    if tui {
                        fail.unwrap().inc(1);
                        total.unwrap().inc(1);
                    }

                    return;
                }
            };

            // deserialize the response to JSON
            spinner.set_message(format!(
                "deserializing [{}] {} reponse",
                &ticker.ticker, &ticker.title
            ));
            let price_response: PriceResponse = match response.json().await {
                Ok(json) => json,
                Err(err) => {
                    error!(
                        "failed to parse Yahoo Finance prices for [{}] {}, error({err})",
                        &ticker.ticker, &ticker.title
                    );

                    if tui {
                        fail.unwrap().inc(1);
                        total.unwrap().inc(1);
                    }

                    return;
                }
            };

            // transform deserialized response
            spinner.set_message(format!("transforming {}", &ticker.ticker));
            let prices = if let Some(data) = price_response.chart.result {
                trace!(
                    "price results found; transforming price data for [{}] {}",
                    &ticker.ticker,
                    &ticker.title
                );

                let base = &data[0];
                let price = &base.indicators.quote[0];
                let adjclose = &base.indicators.adjclose[0].adjclose;
                let timestamps = &base.timestamp;
                let prices = stream::iter(
                    price
                        .open
                        .iter()
                        .zip(price.high.iter())
                        .zip(price.low.iter())
                        .zip(price.close.iter())
                        .zip(price.volume.iter())
                        .zip(adjclose.iter())
                        .zip(timestamps.iter()),
                )
                .then(
                    |((((((open, high), low), close), volume), adj_close), timestamp)| async move {
                        Price {
                            stock_pk: ticker.pk,
                            time: chrono::DateTime::from_timestamp(*timestamp, 0)
                                .expect("invalid timestamp"),
                            interval_pk: 3,
                            open: *open,
                            high: *high,
                            low: *low,
                            close: *close,
                            adj_close: *adj_close,
                            volume: *volume,
                        }
                    },
                )
                .collect::<Vec<Price>>()
                .await;

                trace!(
                    "price data transformation succesful for [{}], {}",
                    &ticker.ticker,
                    &ticker.title
                );
                Prices(prices)
            } else {
                error!(
                        "failed to parse Yahoo Finance prices for [{}] {}, error(no results found within http response)",
                        &ticker.ticker, &ticker.title
                    );

                if tui {
                    fail.unwrap().inc(1);
                    total.unwrap().inc(1);
                }

                return;
            };

            // insert price data to pg
            spinner.set_message(format!(
                "waiting to insert [{}] {}",
                &ticker.ticker, &ticker.title
            ));
            let mut pg_client = pool
                .get()
                .await
                .map_err(|err| {
                    error!("failed to get pg client from pool, error({err})");
                    err
                })
                .unwrap();
            spinner.set_message(format!("inserting [{}] {}", &ticker.ticker, &ticker.title));

            match prices
                .insert(&mut pg_client, &ticker.pk, &ticker.ticker, &ticker.title)
                .await
            {
                Ok(_) => {
                    trace!(
                        "price data inserted successfully for [{}] {}",
                        &ticker.ticker,
                        &ticker.title
                    );

                    if tui {
                        success.unwrap().inc(1);
                        total.unwrap().inc(1);
                    }
                }
                Err(err) => {
                    error!(
                        "failed to insert price data for [{}] {}, error({err})",
                        &ticker.ticker, &ticker.title
                    );

                    if tui {
                        fail.unwrap().inc(1);
                        total.unwrap().inc(1);
                    }

                    return;
                }
            }

            spinner.set_message(format!("{} collected", &ticker.ticker));
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
        println!("collecting stock prices ... done")
    }

    Ok(())
}

#[derive(Debug)]
struct Ticker {
    pk: i32,
    ticker: String,
    title: String,
}

// de
// ----------------------------------------------------------------------------

// output
#[derive(Debug)]
struct Prices(Vec<Price>);

#[derive(Debug)]
struct Price {
    #[allow(dead_code)]
    stock_pk: i32,
    time: chrono::DateTime<chrono::Utc>,
    interval_pk: i16,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    adj_close: f64,
    volume: i64,
}

impl Prices {
    /// INSERT self to pg rows.
    async fn insert(
        &self,
        pg_client: &mut PgClient,
        stock_pk: &i32,
        ticker: &String,
        title: &String,
    ) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&sql::INSERT_PRICE).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(&self.0);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                match transaction
                    .execute(
                        &query,
                        &[
                            &stock_pk,
                            &cell.time,
                            &cell.interval_pk,
                            &cell.open,
                            &cell.high,
                            &cell.low,
                            &cell.close,
                            &cell.adj_close,
                            &cell.volume,
                        ],
                    )
                    .await
                {
                    Ok(_) => trace!("price row inserted for [{ticker}] {title}"),
                    Err(err) => {
                        error! {"failed to insert price rowfor [{ticker}] {title},  error({err})"}
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
                    "failed to commit transaction for Yahoo Finance prices, for [{ticker}] {title}, error({err})"
                );
                err
            })?;

        debug!(
            "[{ticker}] {title} priceset inserted. {}",
            crate::time_elapsed(time)
        );

        Ok(())
    }

    // COPY self to pg rows.
    // async fn copy(
    //     &self,
    //     pg_client: &mut PgClient,
    //     stock_pk: &i32,
    //     ticker: &String,
    //     title: &String,
    // ) -> anyhow::Result<()> {
    //     Ok(())
    // }
}

// input
#[derive(Debug, Deserialize)]
struct PriceResponse {
    chart: Chart,
    // error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Chart {
    result: Option<Vec<Result>>,
}

#[derive(Debug, Deserialize)]
struct Result {
    timestamp: Vec<i64>,
    indicators: Indicators,
}

#[derive(Debug, Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
    adjclose: Vec<AdjClose>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    volume: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct AdjClose {
    adjclose: Vec<f64>,
}

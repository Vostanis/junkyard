use crate::stock::common::convert_date_type;
use crate::stock::sql;
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

// scrape
// -------------------------------------------------------------------------------------------------
pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    // wait for a pg client from the pool
    let pg_client = pool.get().await.map_err(|err| {
        error!("failed to get pg client from pool, error({err})");
        err
    })?;

    // return all tickers from the database
    if tui {
        println!(
            "{bar}\n{name:^40}\n{bar}",
            bar = "=".repeat(40),
            name = "SEC Metrics"
        )
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
    info!("fetching stock.tickers ...");
    let tickers: Vec<Ticker> = pg_client
        .query("SELECT pk, cik, ticker, title FROM stock.tickers", &[])
        .await
        .map_err(|err| {
            error!("failed to fetch stock.tickers, error({err})");
            err
        })?
        .into_iter()
        .map(|row| Ticker {
            pk: row.get(0),
            cik: row.get(1),
            ticker: row.get(2),
            title: row.get(3),
        })
        .collect();
    pb.finish_with_message("fetching tickers ... done");

    drop(pg_client);

    // progress bar
    let (multi, total, success, fail) = if tui {
        crate::tui::multi_progress(tickers.len())?
    } else {
        (None, None, None, None)
    };

    info!("fetching SEC metrics ...");
    let stream = stream::iter(tickers);
    stream
        .for_each_concurrent(num_cpus::get(), |ticker| {
            let time = std::time::Instant::now();

            // progress bars
            let multi = multi.clone();
            let total = total.clone();
            let success = success.clone();
            let fail = fail.clone();
            async move {
                // if tui is enabled, create a progress bar, per task currently being executed
                let spinner = multi.unwrap_or_default().add(
                    ProgressBar::new_spinner()
                        .with_message(format!(
                            "collecting stock metrics for [{symbol}] {title}",
                            symbol = &ticker.ticker,
                            title = &ticker.title
                        ))
                        .with_style(
                            ProgressStyle::default_spinner()
                                .template("\t   > {msg}")
                                .expect("failed to set spinner style"),
                        ),
                );
                spinner.enable_steady_tick(Duration::from_millis(50));

                // construct the path
                let path = format!("./buffer/metrics/CIK{}.json", &ticker.cik);

                // read the file
                trace!("reading file at path: \"{path}\"");
                spinner.set_message(format!(
                    "deserializing metrics for [{}] {}",
                    &ticker.ticker, &ticker.title
                ));
                let json: Facts = match crate::fs::read_json(&path).await {
                    Ok(json) => json,
                    Err(err) => {
                        error!(
                            "failed to read file at \"{path}\" for [{}] {}: {err}",
                            &ticker.ticker, &ticker.title
                        );
                        return;
                    }
                };

                // reformat the data
                trace!(
                    "reformatting facts dataset for [{}] {}",
                    &ticker.ticker,
                    &ticker.title
                );
                spinner.set_message(format!(
                    "reformatting metrics for [{}] {}",
                    &ticker.ticker, &ticker.title
                ));
                let mut metrics: Vec<Metric> = vec![];
                for (dataset_id, metric) in json.facts {
                    // match the dataset_id to any valid datasets included in the static ACC_STDS
                    // map
                    match sql::ACC_STDS.get(&dataset_id) {
                        Some(acc_pk) => {
                            // if valid, parse the dataset
                            for (metric_name, dataset) in metric {
                                spinner.set_message(format!(
                                    "waiting to fetch Primary Keys for [{}] {}",
                                    &ticker.ticker, &ticker.title
                                ));
                                let pg_client = match pool.get().await {
                                    Ok(client) => client,
                                    Err(err) => {
                                        error!("failed to get pg client from pool, error({err})");
                                        if tui {
                                            fail.expect("failbar should have unwrapped").inc(1);
                                            total.expect("totalbar should have unwrapped").inc(1);
                                        }
                                        return;
                                    }
                                };

                                // insert the metric name
                                match pg_client
                                    .query(sql::INSERT_METRIC_PK, &[&metric_name])
                                    .await
                                {
                                    Ok(_) => trace!(
                                        "inserted metric {metric_name} from [{}], {}",
                                        &ticker.ticker,
                                        &ticker.title
                                    ),
                                    Err(err) => {
                                        error!(
                                            "failed to insert metric into metrics_lib, error({err})"
                                        );
                                        if tui {
                                            fail.expect("failbar should have unwrapped").inc(1);
                                            total.expect("totalbar should have unwrapped").inc(1);
                                        }
                                        return;
                                    }
                                };

                                // return the pk for the metric name
                                let metric_pk = match pg_client
                                    .query(sql::GET_METRIC_PK, &[&metric_name])
                                    .await
                                {
                                    Ok(rows) => rows[0].get(0),
                                    Err(err) => {
                                        error!(
                                            "failed to get metric_pk from metrics_lib, error({err})"
                                        );
                                        if tui {
                                            fail.expect("failbar should have unwrapped").inc(1);
                                            total.expect("totalbar should have unwrapped").inc(1);
                                        }
                                        return;
                                    }
                                };

                                drop(pg_client);

                                spinner.set_message(format!(
                                    "reformatting data for [{}] {}",
                                    &ticker.ticker, &ticker.title
                                ));
                                for (_units, values) in dataset.units {
                                    for cell in values {
                                        metrics.push(Metric {
                                            symbol_pk: ticker.pk,
                                            metric_pk,
                                            acc_pk: *acc_pk,
                                            dated: convert_date_type(&cell.dated)
                                                .expect("failed to convert date type"),
                                            val: cell.val,
                                        });
                                    }
                                }
                            }
                        }
                        None => {
                            warn!("unexpected dataset found in Company Fact data {dataset_id}")
                        }
                    };
                }

                // insert into stock.metrics
                spinner.set_message(format!(
                    "waiting to insert metrics for [{}] {}",
                    &ticker.ticker, &ticker.title
                ));
                let mut pg_client = match pool.get().await {
                    Ok(client) => client,
                    Err(err) => {
                        error!("failed to get pg client from pool, error({err})");
                        if tui {
                            fail.expect("failbar should have unwrapped").inc(1);
                            total.expect("totalbar should have unwrapped").inc(1);
                        }
                        return;
                    }
                };

                let query = pg_client
                    .prepare(sql::INSERT_METRIC)
                    .await
                    .expect("failed to prepare INSERT_METRIC statement");

                let transaction = match pg_client.transaction().await {
                    Ok(tr) => Arc::new(tr),
                    Err(err) => {
                        error!(
                            "failed to create transaction for [{}] {}, error({err})",
                            &ticker.ticker, &ticker.title
                        );
                        if tui {
                            fail.expect("failbar should have unwrapped").inc(1);
                            total.expect("totalbar should have unwrapped").inc(1);
                        }
                        return;
                    }
                };

                // stream the data to transaction
                info!(
                    "inserting reformatted metric data for [{}] {}",
                    &ticker.ticker, &ticker.title
                );
                spinner.set_message(format!(
                    "inserting metrics for [{}] {}",
                    &ticker.ticker, &ticker.title
                ));
                let mut stream = stream::iter(metrics);
                while let Some(metric) = stream.next().await {
                    let query = query.clone();
                    let transaction = transaction.clone();
                    let tkr = &ticker.ticker;
                    let title = &ticker.title;

                    // progress bar
                    let total = total.clone();
                    let fail = fail.clone();
                    async move {
                        match transaction
                            .execute(
                                &query,
                                &[
                                    &metric.symbol_pk,
                                    &metric.metric_pk,
                                    &metric.acc_pk,
                                    &metric.dated,
                                    &metric.val,
                                ],
                            )
                            .await
                        {
                            Ok(_) => {
                                trace!("inserted metric data for [{tkr}] {title}");
                            }
                            Err(err) => {
                                error!(
                                    "failed to insert metric data for [{tkr}] {title}, error({err})"
                                );
                                if tui {
                                    fail.expect("failbar should have unwrapped").inc(1);
                                    total.expect("totalbar should have unwrapped").inc(1);
                                }
                                return;
                            }
                        }
                    }
                    .await;
                }

                Arc::into_inner(transaction)
                    .expect("failed to unpack transaction")
                    .commit()
                    .await
                    .expect("failed to commit transaction");

                debug!(
                    "metric data inserted for [{}] {}, {}",
                    &ticker.ticker,
                    &ticker.title,
                    crate::time_elapsed(time)
                );
                if tui {
                    success.expect("successbar should have unwrapped").inc(1);
                    total.expect("totalbar should have unwrapped").inc(1);
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
        println!("collecting stock metrics ... done\n");
    }

    Ok(())
}

struct Ticker {
    pk: i32,
    cik: String,
    ticker: String,
    title: String,
}

// de
// -------------------------------------------------------------------------------------------------

// ======
// Output
// ======

// [
//      {
//          "dated": "2021-01-01",      # types | rust: NaiveDate -> postgres: DATE
//          "metric": "Revenues",
//          "val": "213123123123"
//      },
//      ...
// ]
#[derive(Debug)]
struct Metric {
    symbol_pk: i32,
    metric_pk: i32,
    acc_pk: i32,
    dated: chrono::NaiveDate,
    val: f64,
}

// =====
// Input
// =====

// {
//    "facts": {
#[derive(Deserialize, Debug)]
struct Facts {
    //                      vvvv == "MetricName"
    facts: HashMap<String, HashMap<String, MetricData>>,
    //          ^^^^  == "dei" or "us-gaap"
}

//          "dei": {
//              EntityCommonStockSharesOutstanding": {
#[derive(Deserialize, Debug)]
struct MetricData {
    units: HashMap<String, Vec<DataCell>>,
    //          ^^^^ == "shares" or "USD"
}
//                  "label":"Entity Common Stock, Shares Outstanding",
//                  "description":"Indicate number of shares or ...",
//                  "units": {

//                      "shares": [  <-- or "USD"

#[derive(Deserialize, Debug)]
struct DataCell {
    #[serde(rename = "end")]
    //                ^^^ "end" is a keyword in PostgreSQL, so it's renamed to "dated"
    dated: String,
    val: f64,
}
//                          {
//                              "end":"2009-06-30",
//                              "val":1545912443,
//                              "accn":"0001104659-09-048013",
//                              "fy":2009,
//                              "fp":"Q2",
//                              "form":"10-Q",
//                              "filed":"2009-08-07",
//                              "frame":"CY2009Q2I"
//                          },
//                          ...
//                      ]
//                  },
//              },
//              ...
//          },
//          "us-gaap": {
//               "label": "Accrued Income Taxes, Current",
//               "description": "Carrying amount as of the balance sheet ...",
//                  "units": {
//                      "USD": [
//                          {
//                              "end": "2007-12-31",
//                              "val": 80406000,
//                              "accn": "0001047469-10-001018",
//                              "fy": 2009,
//                              "fp": "FY",
//                              "form": "10-K",
//                              "filed": "2010-02-19",
//                              "frame": "CY2007Q4I"
//                          },
//                          ...
//                      ]
//                }
//          }
//      }
// }

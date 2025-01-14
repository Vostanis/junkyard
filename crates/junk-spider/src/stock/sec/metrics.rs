use crate::key_tracker::KeyTracker;
use crate::stock::common::convert_date_type;
use crate::stock::sql;
use deadpool_postgres::Pool;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use ordered_float::OrderedFloat;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

// scrape
// -------------------------------------------------------------------------------------------------
pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    // wait for a pg client from the pool
    let mut pg_client = pool.get().await.map_err(|err| {
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
        .into_par_iter()
        .map(|row| Ticker {
            pk: row.get(0),
            cik: row.get(1),
            ticker: row.get(2),
            title: row.get(3),
        })
        .collect();
    pb.finish_with_message("fetching tickers ... done");

    // fetch established metrics
    let metrics = KeyTracker::<i32, String>::pg_fetch(
        &mut pg_client,
        "SELECT pk, metric FROM stock.metrics_lib",
    )
    .await;
    let metrics = Arc::new(Mutex::new(metrics));

    // fetch established accounting standards
    let stds = KeyTracker::<i32, String>::pg_fetch(
        &mut pg_client,
        "SELECT pk, accounting FROM stock.acc_stds",
    )
    .await;
    let stds = Arc::new(Mutex::new(stds));

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

            // trackers
            let metrics = metrics.clone();
            let stds = stds.clone();

            // progress bars
            let multi = multi.clone();
            let total = total.clone();
            let success = success.clone();
            let fail = fail.clone();
            async move {
                // if tui is enabled, create a progress bar, per task currently being executed
                let spinner = crate::tui::multi_progress_spinner(
                    multi,
                    format!(
                        "collecting stock metrics for [{symbol}] {title}",
                        symbol = &ticker.ticker,
                        title = &ticker.title
                    ),
                );
                spinner.enable_steady_tick(Duration::from_millis(50));

                // construct the path
                let path = format!("./buffer/metrics/CIK{}.json", &ticker.cik);

                // read the file
                trace!("reading file at path: \"{path}\"");
                if tui {
                    spinner.set_message(format!(
                        "deserializing metrics for [{}] {}",
                        &ticker.ticker, &ticker.title
                    ));
                }
                let json: Facts = match crate::fs::read_json(&path).await {
                    Ok(json) => json,
                    Err(err) => {
                        error!(
                            "failed to read file at \"{path}\" for [{}] {}: {err}",
                            &ticker.ticker, &ticker.title
                        );

                        if tui {
                            fail.expect("failed to unwrap failbar").inc(1);
                            total.expect("failed to unwrap totalbar").inc(1);
                        }

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
                let tbl: Arc<Mutex<HashSet<Metric>>> = Arc::new(Mutex::new(HashSet::new()));
                stream::iter(json.facts)
                    .for_each(|(acc_std, datasets)| {
                        let metrics = metrics.clone();
                        let tbl = tbl.clone();
                        let stds = stds.clone();

                        async move {
                            // track the Accounting Standards PK
                            let std_pk = {
                                let mut stds = stds.lock().await;
                                stds.transact(acc_std)
                            };

                            stream::iter(datasets)
                                .for_each(|(metric, data)| {
                                    let metrics = metrics.clone();
                                    let tbl = tbl.clone();

                                    async move {
                                        // track the Metric Name PK
                                        let metric_pk = {
                                            let mut metrics = metrics.lock().await;
                                            metrics.transact(metric)
                                        };

                                        stream::iter(data.units)
                                            .for_each(|(_units, cells)| {
                                                let tbl = tbl.clone();
                                                async move {
                                                    for cell in cells {
                                                        tbl.lock().await.insert(Metric {
                                                            symbol_pk: ticker.pk,
                                                            metric_pk,
                                                            acc_pk: std_pk,
                                                            dated: convert_date_type(&cell.dated)
                                                                .expect(
                                                                    "failed to convert date type",
                                                                ),
                                                            val: OrderedFloat(cell.val),
                                                        });
                                                    }
                                                }
                                            })
                                            .await;
                                    }
                                })
                                .await;
                        }
                    })
                    .await;

                // get a client back from the pool
                if tui {
                    spinner.set_message(format!(
                        "waiting to insert metrics for [{}] {}",
                        &ticker.ticker, &ticker.title
                    ));
                }
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

                // get the pre-existing data, and remove it from the set
                let exists: HashSet<Metric> = pg_client
                    .query(
                        "
                        SELECT symbol_pk, metric_pk, acc_pk, dated, val FROM stock.metrics
                        WHERE symbol_pk = $1
                    ",
                        &[&ticker.pk],
                    )
                    .await
                    .expect("failed to fetch existing metrics")
                    .iter()
                    .map(|row| Metric {
                        symbol_pk: row.get(0),
                        metric_pk: row.get(1),
                        acc_pk: row.get(2),
                        dated: row.get(3),
                        val: OrderedFloat(row.get(4)),
                    })
                    .collect();

                let tbl: HashSet<Metric> = Arc::into_inner(tbl)
                    .expect("failed to unwrap tbl")
                    .into_inner()
                    .into_iter()
                    .filter(|row| !exists.contains(row))
                    .collect();

                drop(exists);

                // copy the remaining data in
                info!(
                    "copying transformed metric data for [{}] {}",
                    &ticker.ticker, &ticker.title
                );
                if tui {
                    spinner.set_message(format!(
                        "copying metrics for [{}] {}",
                        &ticker.ticker, &ticker.title
                    ));
                }
                match pg_copy(&mut pg_client, tbl).await {
                    Ok(_) => {
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
                    Err(err) => {
                        error!(
                            "failed to copy metrics data for [{}] {}, error({err})",
                            &ticker.ticker, &ticker.title
                        );
                        if tui {
                            fail.expect("failbar should have unwrapped").inc(1);
                            total.expect("totalbar should have unwrapped").inc(1);
                        }
                        return;
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

    let mut pg_client = pool.get().await?;

    Arc::into_inner(stds)
        .expect("failed to unwrap stds")
        .into_inner()
        .pg_insert(
            &mut pg_client,
            "INSERT INTO stock.acc_stds (pk, accounting) VALUES ($1, $2)",
        )
        .await;

    Arc::into_inner(metrics)
        .expect("failed to unwrap metrics")
        .into_inner()
        .pg_insert(
            &mut pg_client,
            "INSERT INTO stock.metrics_lib (pk, metric) VALUES ($1, $2)",
        )
        .await;

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
//          "dated": "2021-01-01",
//          "metric": "Revenues",
//          "val": "213123123123"
//      },
//      ...
// ]
#[derive(Debug, Hash, PartialEq, Eq)]
struct Metric {
    symbol_pk: i32,
    metric_pk: i32,
    acc_pk: i32,
    dated: chrono::NaiveDate,
    val: OrderedFloat<f64>,
}

use crate::http::*;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::Type;

/// Execute the COPY statement, inserting the data into the database.
///
/// ### **NOTE**
///
/// Any TRANSACTION will fail completely if a single COPY statement fails;
/// data must be filtered before being inserted.
async fn pg_copy(pg_client: &mut PgClient, metrics: HashSet<Metric>) -> anyhow::Result<()> {
    let tx = pg_client.transaction().await?;

    let sink = tx.copy_in(sql::COPY_METRIC).await?;
    let writer = BinaryCopyInWriter::new(
        sink,
        &[Type::INT4, Type::INT4, Type::INT4, Type::DATE, Type::FLOAT8],
    );
    futures::pin_mut!(writer);

    for x in metrics {
        match writer
            .as_mut()
            .write(&[
                &x.symbol_pk,
                &x.metric_pk,
                &x.acc_pk,
                &x.dated as &(dyn tokio_postgres::types::ToSql + Sync),
                &x.val.into_inner(),
            ])
            .await
        {
            Ok(_) => (),
            Err(err) => trace!("failed to write metric data to COPY statement, error({err})"),
        }
    }

    writer.finish().await?;
    tx.commit().await?;

    Ok(())
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
    //             ^^^^  == "dei" or "us-gaap"
}

//          "dei": {
//              EntityCommonStockSharesOutstanding": {
#[derive(Deserialize, Debug)]
struct MetricData {
    units: HashMap<String, Vec<DataCell>>,
    //             ^^^^ == "shares" or "USD"
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

use crate::http::*;
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
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::Type;
use tracing::{debug, error, info, trace};

/// The process for copying SEC metric.json files from the /buffer/ directory into the database.
struct Process {
    tickers: Vec<Ticker>,
    metrics: Arc<Mutex<KeyTracker<i32, String>>>,
    acc_stds: Arc<Mutex<KeyTracker<i32, String>>>,
}

impl Process {
    async fn start(pool: &Pool) -> Self {
        // wait for a pg client from the pool
        let mut pg_client = pool.get().await.expect("failed to get pg client from pool");

        // return all tickers from the database
        trace!("fetching tickers ...");
        let tickers: Vec<Ticker> = pg_client
            .query(
                "SELECT pk, file_code, symbol, title FROM stock.symbols",
                &[],
            )
            .await
            .expect("failed to fetch stock.tickers")
            .into_par_iter()
            .map(|row| Ticker {
                pk: row.get(0),
                cik: row.get(1),
                ticker: row.get(2),
                title: row.get(3),
            })
            .collect();

        // fetch established metrics
        trace!("fetching metrics lib ...");
        let metrics = KeyTracker::<i32, String>::pg_fetch(
            &mut pg_client,
            "SELECT pk, metric FROM stock.metrics_lib",
        )
        .await;
        let metrics = Arc::new(Mutex::new(metrics));

        // fetch established accounting standards
        trace!("fetching accounting standards ...");
        let acc_stds = KeyTracker::<i32, String>::pg_fetch(
            &mut pg_client,
            "SELECT pk, accounting FROM stock.acc_stds",
        )
        .await;
        drop(pg_client);
        let acc_stds = Arc::new(Mutex::new(acc_stds));

        Self {
            tickers,
            metrics,
            acc_stds,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////
// -- SCRAPE --
////////////////////////////////////////////////////////////////////////////////////////////

pub async fn scrape(pool: &Pool, tui: bool) -> anyhow::Result<()> {
    // return all tickers from the database
    if tui {
        println!(
            "{bar}\n{name:^40}\n{bar}",
            bar = "=".repeat(40),
            name = "SEC Metrics"
        )
    }

    debug!("starting metric scraping process ...");
    let pb = if tui {
        let pb = ProgressBar::new_spinner()
            .with_message("initialising tables ...")
            .with_style(
                ProgressStyle::default_spinner()
                    .template("{msg} {spinner:.magenta}")
                    .expect("failed to set progress bar style"),
            );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    } else {
        ProgressBar::hidden()
    };
    let pr = Process::start(pool).await;
    pb.finish_and_clear();
    if tui {
        println!("initialising tables ... done");
    }

    // progress bar
    let (multi, total, success, fail) = if tui {
        crate::tui::multi_progress(pr.tickers.len())?
    } else {
        (None, None, None, None)
    };

    let stream = stream::iter(pr.tickers);
    stream
        .for_each_concurrent(num_cpus::get(), |ticker| {
            let time = std::time::Instant::now();

            // trackers
            let metrics = pr.metrics.clone();
            let stds = pr.acc_stds.clone();

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

                // build a table of unique rows (and keep everything async)
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

                                        let cells = data
                                            .units
                                            .into_iter()
                                            .flat_map(|(_units, cells)| cells)
                                            .collect::<Vec<_>>();

                                        let mut batch = vec![];
                                        for cell in cells {
                                            batch.push(Metric {
                                                symbol_pk: ticker.pk,
                                                metric_pk,
                                                acc_pk: std_pk,
                                                start_date: {
                                                    if let Some(start_date) = cell.start_date {
                                                        Some(convert_date_type(&start_date)
                                                            .expect("failed to convert date type"))
                                                    } else { 
                                                        None 
                                                    }
                                                },
                                                end_date: convert_date_type(&cell.end_date)
                                                    .expect("failed to convert date type"),
                                                filing_date: convert_date_type(&cell.filing_date)
                                                    .expect("failed to convert date type"),
                                                year: cell.fy,
                                                period: cell.fp,
                                                form: cell.form,
                                                val: OrderedFloat(cell.val),
                                                accn: cell.accn,
                                                frame: cell.frame,
                                            });
                                        }

                                        let mut tbl = tbl.lock().await;
                                        for metric in batch {
                                            tbl.insert(metric);
                                        }
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
                if tui {
                    spinner.set_message(format!(
                        "removing pre-existing Primary Keys for [{}] {}",
                        &ticker.ticker, &ticker.title
                    ));
                }
                let exists: HashSet<MetricPrimaryKey> = pg_client
                    .query(
                        "
                        SELECT symbol_pk, metric_pk, acc_pk, end_date, filing_date, year, period, form, val, accn FROM stock.metrics
                        WHERE symbol_pk = $1
                    ",
                        &[&ticker.pk],
                    )
                    .await
                    .expect("failed to fetch existing metrics")
                    .iter()
                    .map(|row| MetricPrimaryKey {
                        symbol_pk: row.get(0),
                        metric_pk: row.get(1),
                        acc_pk: row.get(2),
                        end_date: row.get(3),
                        filing_date: row.get(4),
                        year: row.get(5),
                        period: row.get(6),
                        form: row.get(7),
                        val: OrderedFloat(row.get(8)),
                        accn: row.get(9),
                    })
                    .collect();

                let tbl: HashSet<Metric> = Arc::into_inner(tbl)
                    .expect("failed to unwrap tbl")
                    .into_inner()
                    .into_iter()
                    .filter(|row| {
                        !exists.contains(&row.pk())
                    })
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

    if tui {
        println!("collecting stock metrics ... done");
    }

    let mut pg_client = pool.get().await?;

    Arc::into_inner(pr.acc_stds)
        .expect("failed to unwrap stds")
        .into_inner()
        .pg_insert(
            &mut pg_client,
            "INSERT INTO stock.acc_stds (pk, accounting) VALUES ($1, $2)
            ON CONFLICT DO NOTHING",
        )
        .await;
    if tui {
        println!("inserted accounting standards");
    }

    Arc::into_inner(pr.metrics)
        .expect("failed to unwrap metrics")
        .into_inner()
        .pg_insert(
            &mut pg_client,
            "INSERT INTO stock.metrics_lib (pk, metric) VALUES ($1, $2)
            ON CONFLICT DO NOTHING",
        )
        .await;

    if tui {
        println!("inserted metric names\n");
    }

    Ok(())
}

struct Ticker {
    pk: i32,
    cik: String,
    ticker: String,
    title: String,
}

/////////////////////////////////////////////////////////////////////////////////////////////
// -- DESERIALIZE --
/////////////////////////////////////////////////////////////////////////////////////////////

// Output
// ======
// [
//      {
//          "symbol_pk": 1,
//          "metric_pk": 2,
//          "acc_pk": 3,
//          "start_date": "2021-01-01",
//          "end_date": "2021-03-30",
//          "filing_date": "2021-01-09",
//          "year": 2021,
//          "period": "Q2",
//          "form": "10-K",
//          "metric": "Revenues",
//          "val": "213123123123",
//          "accn": "324987349-12321-2109381283",
//          "frame": "CY2021Q2",
//      },
//      ...
// ]
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Metric {
    pub symbol_pk: i32,
    pub metric_pk: i32,
    pub acc_pk: i32,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: chrono::NaiveDate,
    pub filing_date: chrono::NaiveDate,
    pub year: i16,
    pub period: String,
    pub form: String,
    pub val: OrderedFloat<f64>,
    pub accn: String,
    pub frame: Option<String>,
}

impl Metric {
    fn pk(&self) -> MetricPrimaryKey {
        MetricPrimaryKey {
            symbol_pk: self.symbol_pk,
            metric_pk: self.metric_pk,
            acc_pk: self.acc_pk,
            end_date: self.end_date,
            filing_date: self.filing_date,
            year: self.year,
            period: self.period.clone(),
            form: self.form.clone(),
            val: self.val,
            accn: self.accn.clone(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct MetricPrimaryKey {
    pub symbol_pk: i32,
    pub metric_pk: i32,
    pub acc_pk: i32,
    pub end_date: chrono::NaiveDate,
    pub filing_date: chrono::NaiveDate,
    pub year: i16,
    pub period: String,
    pub form: String,
    pub val: OrderedFloat<f64>,
    pub accn: String,
}

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
        &[
            Type::INT4,
            Type::INT4,
            Type::INT4,
            Type::DATE,
            Type::DATE,
            Type::DATE,
            Type::INT2,
            Type::BPCHAR,
            Type::VARCHAR,
            Type::FLOAT8,
            Type::VARCHAR,
            Type::VARCHAR,
        ],
    );
    futures::pin_mut!(writer);

    for x in metrics {
        match writer
            .as_mut()
            .write(&[
                &x.symbol_pk,
                &x.metric_pk,
                &x.acc_pk,
                &x.start_date as &(dyn tokio_postgres::types::ToSql + Sync),
                &x.end_date as &(dyn tokio_postgres::types::ToSql + Sync),
                &x.filing_date as &(dyn tokio_postgres::types::ToSql + Sync),
                &x.year,
                &x.period,
                &x.form,
                &x.val.into_inner(),
                &x.accn,
                &x.frame,
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

// Input
// =====
//
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
    #[serde(rename = "start")]
    start_date: Option<String>,
    #[serde(rename = "end")]
    end_date: String,
    #[serde(rename = "filed")]
    filing_date: String,
    val: f64,
    fy: i16,
    fp: String,
    form: String,
    accn: String,
    frame: Option<String>,
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


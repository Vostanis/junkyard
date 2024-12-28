use crate::http::*;
use crate::stock::common::convert_date_type;
use crate::stock::sql;
use futures::{stream, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace, warn};

// scrape
// -------------------------------------------------------------------------------------------------
pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
    // return all tickers from the database
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

    let pg_client = Arc::new(Mutex::new(pg_client));

    info!("fetching SEC metrics ...");
    let stream = stream::iter(tickers);
    stream
        .for_each_concurrent(12, |ticker| {
            let time = std::time::Instant::now();

            // create 3 clones of the pg_client for the 3 processes
            let pk_insert_pg_client = pg_client.clone();
            let pk_get_pg_client = pg_client.clone();
            let pg_client = pg_client.clone();
            async move {
                // construct the path
                let path = format!("./buffer/metrics/CIK{}.json", &ticker.cik);

                // read the file
                trace!("reading file at path: \"{path}\"");
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
                let mut metrics: Vec<Metric> = vec![];
                for (dataset_id, metric) in json.facts {
                    // match the dataset_id to any valid datasets included in the static ACC_STDS
                    // map
                    match sql::ACC_STDS.get(&dataset_id) {
                        Some(acc_pk) => {
                            // if valid, parse the dataset
                            for (metric_name, dataset) in metric {
                                // insert the metric name
                                match pk_insert_pg_client
                                    .lock()
                                    .await
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
                                        return;
                                    }
                                };

                                // return the pk for the metric name
                                let metric_pk = match pk_get_pg_client
                                    .lock()
                                    .await
                                    .query(sql::GET_METRIC_PK, &[&metric_name])
                                    .await
                                {
                                    Ok(rows) => rows[0].get(0),
                                    Err(err) => {
                                        error!(
                                            "failed to get metric_pk from metrics_lib, error({err})"
                                        );
                                        return;
                                    }
                                };

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
                let mut pg_client = pg_client.lock().await;
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
                        return;
                    }
                };

                // stream the data to transaction
                info!(
                    "inserting reformatted metric data for [{}] {}",
                    &ticker.ticker, &ticker.title
                );
                let mut stream = stream::iter(metrics);
                while let Some(metric) = stream.next().await {
                    let query = query.clone();
                    let transaction = transaction.clone();
                    let tkr = &ticker.ticker;
                    let title = &ticker.title;
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
                            Ok(_) => trace!("inserted metric data for [{tkr}] {title}"),
                            Err(err) => {
                                error!(
                                    "failed to insert metric data for [{tkr}] {title}, error({err})"
                                );
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
            }
        })
        .await;

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

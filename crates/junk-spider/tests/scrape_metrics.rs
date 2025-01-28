use chrono::NaiveDate;
use futures::{stream, StreamExt};
use junk_spider::fs::read_json;
use junk_spider::key_tracker::KeyTracker;
use ordered_float::OrderedFloat;
use serde::Deserialize;
use std::collections::{HashMap as Map, HashSet as Set};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn scrape_metrics() {
    // -- OPEN PG CONNECTION --
    dotenv::dotenv().ok();
    let (mut pg_client, pg_conn) =
        tokio_postgres::connect(&dotenv::var("FINDUMP_URL").unwrap(), tokio_postgres::NoTls)
            .await
            .unwrap();
    tokio::spawn(async move {
        if let Err(e) = pg_conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    // -- FETCH METRICS --
    let time = std::time::Instant::now();
    let metrics = KeyTracker::<i32, String>::pg_fetch(
        &mut pg_client,
        "SELECT pk, metric FROM stock.metrics_lib",
    )
    .await;
    let metrics = Arc::new(Mutex::new(metrics));

    // -- FETCH ACCOUNTING STANDARDS --
    let stds = KeyTracker::<i32, String>::pg_fetch(
        &mut pg_client,
        "SELECT pk, accounting FROM stock.acc_stds",
    )
    .await;
    let stds = Arc::new(Mutex::new(stds));

    // -- FETCH TICKERS --
    let mut tickers = KeyTracker::<i32, String>::pg_fetch(
        &mut pg_client,
        "SELECT pk, cik FROM stock.tickers WHERE pk = 1087",
    )
    .await;
    println!("FETCH ARTIFACTS: {:?}", time.elapsed().as_secs_f64());

    // -- READ FILE --
    let time = std::time::Instant::now();
    let json: Facts = read_json("./tests/files/metrics.json").await.unwrap();
    let cik = format!("{:010}", json.cik);
    assert_eq!(json.cik, 750004);
    println!("READ FILE: {:?}s", time.elapsed().as_secs_f64());

    // -- FIND TICKER --
    let symbol_pk = tickers.transact(cik);

    // -- TRANSFORMATION --
    let time = std::time::Instant::now();
    let raw_tbl: Arc<Mutex<Vec<Metric>>> = Arc::new(Mutex::new(vec![]));

    // stream over the data
    stream::iter(json.facts)
        .for_each_concurrent(num_cpus::get(), |(std, datasets)| {
            let metrics = metrics.clone();
            let raw_tbl = raw_tbl.clone();
            let stds = stds.clone();

            async move {
                // add accounting standard to the set
                let std_pk = {
                    let mut stds = stds.lock().await;
                    stds.transact(std)
                };

                // continue the stream
                stream::iter(datasets)
                    .for_each(|(metric, data)| {
                        let metrics = metrics.clone();
                        let raw_tbl = raw_tbl.clone();

                        async move {
                            let metric_pk = {
                                // add metric to the set
                                let mut metrics = metrics.lock().await;
                                metrics.transact(metric)
                            };

                            stream::iter(data.units)
                                .for_each(|(_units, values)| {
                                    let raw_tbl = raw_tbl.clone();

                                    async move {
                                        stream::iter(values)
                                            .for_each(|row| {
                                                let raw_tbl = raw_tbl.clone();
                                                // raw data push happens here
                                                async move {
                                                    let mut raw_tbl = raw_tbl.lock().await;
                                                    raw_tbl.push(Metric {
                                                        // pk
                                                        symbol_pk,
                                                        metric_pk,
                                                        std_pk,

                                                        // raw
                                                        dated: convert_date_type(&row.dated)
                                                            .unwrap(),
                                                        val: OrderedFloat(row.val),
                                                    })
                                                }
                                            })
                                            .await;
                                    }
                                })
                                .await;
                        }
                    })
                    .await;
            }
        })
        .await;
    println!(
        "TRANSFORM DATASET: {:?}s, count: {}",
        time.elapsed().as_secs_f64(),
        raw_tbl.lock().await.len()
    );

    // -- SELECT ALL PRIMARY KEYS FROM THE ACTUAL stock.metrics --
    let time = std::time::Instant::now();
    let exists: Set<Metric> = pg_client
        .query(
            "
            SELECT symbol_pk, metric_pk, acc_pk, dated, val FROM stock.metrics
            WHERE symbol_pk = $1
        ",
            &[&symbol_pk],
        )
        .await
        .unwrap()
        .iter()
        .map(|row| Metric {
            symbol_pk: row.get(0),
            metric_pk: row.get(1),
            std_pk: row.get(2),
            dated: row.get(3),
            val: OrderedFloat(row.get(4)),
        })
        .collect();
    println!(
        "SELECT *: {:?}s, count: {}",
        time.elapsed().as_secs_f64(),
        exists.len()
    );

    // -- FILTER EXISTING FROM COLLECTED --
    let time = std::time::Instant::now();
    let raw_tbl = Arc::into_inner(raw_tbl).unwrap().into_inner();
    let filtered: Vec<Metric> = raw_tbl
        .into_iter()
        .filter(|cell| !exists.contains(&cell))
        .collect();
    println!(
        "FILTERED PRICES: {:?}s, count: {}",
        time.elapsed().as_secs_f64(),
        filtered.len()
    );
}

///////////////////////////////////////////////////////////////////////

// -- SERIALIZATION --
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Hash)]
struct Metric {
    symbol_pk: i32,
    metric_pk: i32,
    std_pk: i32,
    dated: NaiveDate,
    val: OrderedFloat<f64>,
}

// -- DESERIALIZATION --
#[derive(Deserialize, Debug)]
struct Facts {
    cik: u32,
    facts: Map<String, Map<String, MetricData>>,
}

#[derive(Deserialize, Debug)]
struct MetricData {
    units: Map<String, Vec<DataCell>>,
}

#[derive(Deserialize, Debug)]
struct DataCell {
    #[serde(rename = "end")]
    dated: String,
    val: f64,
}

// -- CONVERSION --
pub fn convert_date_type(str_date: &String) -> anyhow::Result<chrono::NaiveDate> {
    let date = chrono::NaiveDate::parse_from_str(&str_date, "%Y-%m-%d")?;
    Ok(date)
}

// use chrono::NaiveDate;
// use futures::{stream, StreamExt};
// use junk_spider::fs::read_json;
// use junk_spider::http::*;
// use junk_spider::key_tracker::KeyTracker;
// use serde::Deserialize;
// use std::collections::{HashMap as Map, HashSet as Set};
// use std::sync::{Arc, Mutex};
//
// #[tokio::test]
// async fn scrape_metrics() {
//     // -- READ FILE --
//     let time = std::time::Instant::now();
//     let json: Facts = read_json("./tests/files/metrics.json").await.unwrap();
//     assert_eq!(json.cik, 750004);
//     println!("READ FILE: {:?}s", time.elapsed().as_secs_f64());
//
//     // -- TRANSFORMATION --
//     let time = std::time::Instant::now();
//     let metrics: Arc<Mutex<Map<String, u64>>> = Arc::new(Mutex::new(Map::new()));
//     let stds: Arc<Mutex<Map<String, u64>>> = Arc::new(Mutex::new(Map::new()));
//     let raw_tbl: Arc<Mutex<Vec<Metric>>> = Arc::new(Mutex::new(vec![]));
//
//     let metric_counter = metrics.clone();
//
//     // stream over the data
//     stream::iter(json.facts)
//         .for_each_concurrent(num_cpus::get(), |(std, datasets)| {
//             let metrics = metrics.clone();
//             let raw_tbl = raw_tbl.clone();
//             let stds = stds.clone();
//
//             async move {
//                 // add std to the set
//                 let mut stds = stds.lock().unwrap();
//                 stds.insert(std.clone());
//
//                 // continue the stream
//                 stream::iter(datasets)
//                     .for_each(|(metric, data)| {
//                         let metrics = metrics.clone();
//                         let raw_tbl = raw_tbl.clone();
//
//                         async move {
//                             // add metric to the set
//                             let mut metrics = metrics.lock().unwrap();
//                             metrics.insert(metric.clone());
//
//                             // continue stream
//                             stream::iter(data.units)
//                                 .for_each(|(_units, values)| {
//                                     let raw_tbl = raw_tbl.clone();
//
//                                     async move {
//                                         stream::iter(values)
//                                             .for_each(|row| {
//                                                 let raw_tbl = raw_tbl.clone();
//
//                                                 // raw data push happens here
//                                                 async move {
//                                                     let mut raw_tbl = raw_tbl.lock().unwrap();
//                                                     raw_tbl.push(Metric {
//                                                         // pk
//                                                         symbol_pk: 1_000_000,
//                                                         metric_pk: 1_000_000,
//                                                         std_pk: 1_000_000,
//
//                                                         // raw
//                                                         dated: convert_date_type(&row.dated)
//                                                             .unwrap(),
//                                                         val: row.val,
//                                                     })
//                                                 }
//                                             })
//                                             .await;
//                                     }
//                                 })
//                                 .await;
//                         }
//                     })
//                     .await;
//             }
//         })
//         .await;
//
//     let raw_tbl = Arc::into_inner(raw_tbl).unwrap().into_inner().unwrap();
//
//     println!("{:?}", raw_tbl);
//     println!(
//         "ASYNC TRANSFORM: {:?}s, count: {}",
//         time.elapsed().as_secs_f64(),
//         raw_tbl.len()
//     );
// }
//
// ///////////////////////////////////////////////////////////////////////
//
// // -- SERIALIZATION --
// #[derive(Debug)]
// struct Metric {
//     symbol_pk: i32,
//     metric_pk: i32,
//     std_pk: i32,
//     dated: NaiveDate,
//     val: f64,
// }
//
// // -- DESERIALIZATION --
// #[derive(Deserialize, Debug)]
// struct Facts {
//     cik: u32,
//     facts: Map<String, Map<String, MetricData>>,
// }
//
// #[derive(Deserialize, Debug)]
// struct MetricData {
//     units: Map<String, Vec<DataCell>>,
// }
//
// #[derive(Deserialize, Debug)]
// struct DataCell {
//     #[serde(rename = "end")]
//     dated: String,
//     val: f64,
// }
//
// // -- CONVERSION --
// pub fn convert_date_type(str_date: &String) -> anyhow::Result<chrono::NaiveDate> {
//     let date = chrono::NaiveDate::parse_from_str(&str_date, "%Y-%m-%d")?;
//     Ok(date)
// }

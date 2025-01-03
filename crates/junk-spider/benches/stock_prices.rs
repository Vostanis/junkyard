#![allow(dead_code)]

use criterion::*;
use futures::{stream, StreamExt};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

// read a json file to a string
#[inline]
fn read_file_to_string(path: String) -> String {
    let mut file = File::open(path)
        .map_err(|err| {
            println!("Unable to open file: {:?}", err);
            err
        })
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    contents
}

// deserialize prices (from a string)
// ----------------------------------------------------------
#[inline]
fn de_prices(to_de: String) -> PriceResponse {
    serde_json::from_str(&to_de).expect("Unable to deserialize")
}

fn benchmark_deserialization(c: &mut Criterion) {
    let path = format!("./benches/files/prices.json");
    let file_contents = read_file_to_string(path);

    c.bench_function("deserialize prices", |b| {
        b.iter(|| {
            let _prices: PriceResponse = de_prices(black_box(file_contents.clone()));
        })
    });
}

// transform prices
// ----------------------------------------------------------
#[inline]
async fn tr_prices(to_tr: PriceResponse) -> anyhow::Result<Vec<Price>> {
    let prices = if let Some(data) = to_tr.chart.result {
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
                    stock_pk: 100,
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
        prices
    } else {
        return Err(anyhow::anyhow!("no data"));
    };

    Ok(prices)
}

fn benchmark_transformation(c: &mut Criterion) {
    let path = format!("./benches/files/prices.json");
    let file_contents = read_file_to_string(path);
    let prices: PriceResponse = de_prices(file_contents);

    c.bench_function("transform prices", |b| {
        b.iter(|| {
            let _transformed_prices = tr_prices(black_box(prices.clone()));
        })
    });
}

// load prices using INSERT statements
#[inline]
async fn load_prices_with_insert(prices: Vec<Price>) -> anyhow::Result<()> {
    for price in prices {
        println!("{:?}", price);
    }

    Ok(())
}

// load prices using COPY statements
#[inline]
async fn load_prices_with_copy(prices: Vec<Price>) -> anyhow::Result<()> {
    for price in prices {
        println!("{:?}", price);
    }

    Ok(())
}

fn benchmark_loading(c: &mut Criterion) {
    let path = format!("./benches/files/prices.json");
    let file_contents = read_file_to_string(path);
    let prices: PriceResponse = de_prices(file_contents);

    c.bench_function("load prices with INSERT", |b| {
        b.iter(|| {
            let _transformed_prices = tr_prices(black_box(prices.clone()));
        })
    });

    c.bench_function("load prices with COPY", |b| {
        b.iter(|| {
            let _transformed_prices = tr_prices(black_box(prices.clone()));
        })
    });
}

criterion_group!(benches, benchmark_deserialization, benchmark_transformation);
criterion_main!(benches);

////////////////////////////////////////////////////////////////////

// Output cell
// ----------------------------------------------------------
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

// Input
// ----------------------------------------------------------
#[derive(Clone, Debug, Deserialize)]
struct PriceResponse {
    chart: Chart,
}

#[derive(Clone, Debug, Deserialize)]
struct Chart {
    result: Option<Vec<Result>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Result {
    timestamp: Vec<i64>,
    indicators: Indicators,
}

#[derive(Clone, Debug, Deserialize)]
struct Indicators {
    quote: Vec<Quote>,
    adjclose: Vec<AdjClose>,
}

#[derive(Clone, Debug, Deserialize)]
struct Quote {
    open: Vec<f64>,
    high: Vec<f64>,
    low: Vec<f64>,
    close: Vec<f64>,
    volume: Vec<i64>,
}

#[derive(Clone, Debug, Deserialize)]
struct AdjClose {
    adjclose: Vec<f64>,
}

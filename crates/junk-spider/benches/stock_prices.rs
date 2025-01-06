#![allow(dead_code)]

use criterion::*;
use futures::{stream, StreamExt};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use tokio_postgres::types::{ToSql, Type};

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
async fn load_prices_with_insert(
    pg_client: &mut tokio_postgres::Client,
    prices: Vec<Price>,
) -> anyhow::Result<()> {
    // start a transaction
    let query = pg_client.prepare("
        INSERT INTO stock.prices (symbol_pk, dt, interval_pk, opening, high, low, closing, adj_close, volume)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (symbol_pk, dt, interval_pk) DO NOTHING
    ").await?;
    let tx = std::sync::Arc::new(pg_client.transaction().await?);

    // iterate over the data stream and execute pg rows
    let placeholder_pk = 999_998;
    let mut stream = stream::iter(&prices);
    while let Some(cell) = stream.next().await {
        let query = query.clone();
        let tx = tx.clone();

        async move {
            tx.execute(
                &query,
                &[
                    &placeholder_pk,
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
            .unwrap()
        }
        .await;
    }

    // unpack the transcation and commit it to the database
    std::sync::Arc::into_inner(tx)
        .expect("failed to unpack Transaction from Arc")
        .commit()
        .await
        .unwrap();

    Ok(())
}

// load prices using COPY statements
#[inline]
async fn load_prices_with_copy(
    pg_client: &mut tokio_postgres::Client,
    prices: Vec<Price>,
) -> anyhow::Result<()> {
    // Start a transaction
    let tx = pg_client.transaction().await?;

    // Prepare the COPY statement
    let copy_stmt = "
        COPY stock.prices (symbol_pk, dt, interval_pk, opening, high, low, closing, adj_close, volume) 
        FROM STDIN WITH (FORMAT binary)
    ";

    // Begin the COPY operation
    let sink = tx.copy_in(copy_stmt).await?;

    // vec![
    //         &placeholder_pk,
    //         &cell.time,
    //         &cell.interval_pk,
    //         &cell.open,
    //         &cell.high,
    //         &cell.low,
    //         &cell.close,
    //         &cell.adj_close,
    //         &cell.volume,
    // ];
    let writer = tokio_postgres::binary_copy::BinaryCopyInWriter::new(
        sink,
        &[
            Type::INT4,
            Type::TIMESTAMPTZ,
            Type::INT2,
            Type::FLOAT4,
            Type::FLOAT4,
            Type::FLOAT4,
            Type::FLOAT4,
            Type::FLOAT4,
            Type::INT8,
        ],
    );
    futures::pin_mut!(writer);

    // Iterate over the data stream and write to the sink in binary format
    let placeholder_pk = 999_999;
    let mut stream = stream::iter(&prices);
    while let Some(cell) = stream.next().await {
        writer
            .as_mut()
            .write(&[
                &placeholder_pk as &(dyn ToSql + Sync),
                &cell.time as &(dyn ToSql + Sync),
                &cell.interval_pk as &(dyn ToSql + Sync),
                &cell.open as &(dyn ToSql + Sync),
                &cell.high as &(dyn ToSql + Sync),
                &cell.low as &(dyn ToSql + Sync),
                &cell.close as &(dyn ToSql + Sync),
                &cell.adj_close as &(dyn ToSql + Sync),
                &cell.volume as &(dyn ToSql + Sync),
            ])
            .await
            .unwrap();
    }

    writer.finish().await?;
    tx.commit().await?;

    Ok(())
}

use criterion::async_executor::FuturesExecutor;
async fn benchmark_loading(c: &mut Criterion) {
    let path = format!("./benches/files/prices.json");
    let file_contents = read_file_to_string(path);
    let prices: PriceResponse = de_prices(file_contents);
    let prices = tr_prices(prices).await.unwrap();

    // open a connection
    let (pg_client, pg_conn) = tokio_postgres::connect(
        &dotenv::var("FINDUMP_URL").expect("environment variable FINDUMP_URL"),
        tokio_postgres::NoTls,
    )
    .await
    .unwrap();

    // hold the connection
    tokio::spawn(async move { if let Err(_err) = pg_conn.await {} });

    // benches
    let pg_clone = pg_client.clone();
    c.bench_function("load prices with INSERT", move |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            load_prices_with_insert(&mut pg_client, black_box(prices.clone())).await
        })
    });

    // c.bench_function("load prices with COPY", |b| {
    //     b.iter(|| {
    //         let _copy_load = load_prices_with_copy(&mut pg_client, black_box(prices.clone()));
    //     })
    // });
}

criterion_group!(benches, benchmark_deserialization, benchmark_transformation,);
criterion_main!(benches);

////////////////////////////////////////////////////////////////////

// Output cell
// ----------------------------------------------------------
#[derive(Clone, Debug)]
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

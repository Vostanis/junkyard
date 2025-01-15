use chrono::{DateTime, Utc};
use dotenv::var;
use futures::{stream, StreamExt};
use junk_spider::http::PgClient;
use serde::Deserialize;
use std::collections::HashSet as Set;
use std::sync::Arc;
use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::{ToSql, Type};
use tokio_postgres::{self as pg, NoTls};

const PLACEHOLDER_PK: i32 = 1_000_000;

#[tokio::test]
async fn scrape_prices() {
    // -- BUILD DEFAULT HTTP CLIENT --
    let client = reqwest::ClientBuilder::new()
        .user_agent("kimonvostanis@gmail.com")
        .build()
        .unwrap();
    dbg!(&client);

    // -- HTTP GET REQUEST & DESERIALIZATION --
    let time = std::time::Instant::now();
    let res = client
        .get("https://query1.finance.yahoo.com/v8/finance/chart/MSFT?symbol=MSFT&interval=1d&range=max&events=div|split|capitalGain")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::OK);
    println!("HTTP GET: {:?}s", time.elapsed().as_secs_f64());

    let time = std::time::Instant::now();
    let de: PriceResponse = res.json().await.unwrap();
    assert!(de.chart.result.is_some());
    println!("DESERIALIZE: {:?}s", time.elapsed().as_secs_f64());

    // -- TRANSFORMATION --
    let time = std::time::Instant::now();
    let prices = if let Some(data) = de.chart.result {
        let base = &data[0];
        let price = &base.indicators.quote[0];
        let adjclose = &base.indicators.adjclose[0].adjclose;
        let timestamps = &base.timestamp;
        stream::iter(
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
                    stock_pk: PLACEHOLDER_PK,
                    time: DateTime::from_timestamp(*timestamp, 0).unwrap(),
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
        .await
    } else {
        panic!("No price data found");
    };
    assert!(!prices.is_empty());
    println!("TRANSFORM: {:?}s", time.elapsed().as_secs_f64());

    // -- CONNECT TO POSTGRES --
    let time = std::time::Instant::now();
    let (mut pg_client, pg_conn) = pg::connect(
        &var("FINDUMP_URL").expect("environment variable FINDUMP_URL"),
        NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(err) = pg_conn.await {
            panic!("findump connection error: {}", err)
        }
    });
    println!("CONNECT: {:?}s", time.elapsed().as_secs_f64());

    println!("\nRunnning PostgreSQL queries ...");

    // -- CREATE TABLE IN POSTGRES --
    let time = std::time::Instant::now();
    pg_client
        .query(
            "
        CREATE TABLE IF NOT EXISTS test.prices (
            symbol_pk INT,
            interval_pk SMALLINT,
            dt TIMESTAMP WITH TIME ZONE NOT NULL,
            opening FLOAT,
            high FLOAT,
            low FLOAT,
            closing FLOAT,
            adj_close FLOAT,
            volume BIGINT,
            PRIMARY KEY (symbol_pk, interval_pk, dt)
        )",
            &[],
        )
        .await
        .unwrap();
    println!("CREATE TABLE: {:?}s", time.elapsed().as_secs_f64());

    let midpoint = prices.len() / 2;

    // -- INSERT INTO POSTGRES --
    let time = std::time::Instant::now();
    insert(prices[..midpoint].to_vec(), &mut pg_client)
        .await
        .unwrap();
    println!("INSERT test.prices: {:?}s", time.elapsed().as_secs_f64());

    // -- TRUNCATE TABLE IN POSTGRES --
    let time = std::time::Instant::now();
    pg_client
        .query("TRUNCATE TABLE test.prices", &[])
        .await
        .unwrap();
    println!("TRUNCATE TABLE: {:?}s", time.elapsed().as_secs_f64());

    // -- SELECT ALL PRIMARY KEYS FROM THE ACTUAL stock.prices --
    let time = std::time::Instant::now();
    let exists: Set<PricePrimaryKey> = pg_client
        .query(
            "
            SELECT p.symbol_pk, p.dt, p.interval_pk FROM stock.prices p 
            WHERE p.symbol_pk = $1
        ",
            &[&PLACEHOLDER_PK],
        )
        .await
        .unwrap()
        .iter()
        .map(|row| PricePrimaryKey {
            stock_pk: row.get(0),
            time: row.get(1),
            interval_pk: row.get(2),
        })
        .collect();
    println!(
        "SELECT *: {:?}s, count: {}",
        time.elapsed().as_secs_f64(),
        exists.len()
    );

    // -- FILTER EXISTING FROM COLLECTED --
    let time = std::time::Instant::now();
    let filtered_prices: Vec<_> = prices
        .into_iter()
        .filter(|cell| {
            !exists.contains(&PricePrimaryKey {
                stock_pk: cell.stock_pk,
                time: cell.time,
                interval_pk: cell.interval_pk,
            })
        })
        .collect();
    println!(
        "filtered prices: {:?}s, count: {}",
        time.elapsed().as_secs_f64(),
        filtered_prices.len()
    );

    // -- COPY FILTERED INTO POSTGRES --
    let time = std::time::Instant::now();
    copy(filtered_prices.clone(), &mut pg_client).await.unwrap();
    println!("COPY test.prices: {:?}s", time.elapsed().as_secs_f64());

    // -- REMOVE TABLE IN POSTGRES --
    let time = std::time::Instant::now();
    pg_client
        .query("DROP TABLE IF EXISTS test.prices", &[])
        .await
        .unwrap();
    println!("DROP TABLE: {:?}s", time.elapsed().as_secs_f64());
}

//////////////////////////////////////////////////////////////////////

// -- SERIALIZATION --
#[derive(Clone, Debug)]
struct Price {
    #[allow(dead_code)]
    stock_pk: i32,
    time: DateTime<Utc>,
    interval_pk: i16,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    adj_close: f64,
    volume: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PricePrimaryKey {
    stock_pk: i32,
    time: DateTime<Utc>,
    interval_pk: i16,
}

// -- DESERIALIZATION --
#[derive(Debug, Deserialize)]
struct PriceResponse {
    chart: Chart,
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

//////////////////////////////////////////////////////////////////////

// -- INSERT INTO POSTGRES --
async fn insert(prices: Vec<Price>, pg_client: &mut PgClient) -> anyhow::Result<()> {
    // preprocess pg query as transaction
    let query = pg_client.prepare("
        INSERT INTO test.prices (symbol_pk, dt, interval_pk, opening, high, low, closing, adj_close, volume)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (symbol_pk, dt, interval_pk) DO NOTHING
    ").await.unwrap();
    let tx = Arc::new(pg_client.transaction().await?);

    // iterate over the data stream and execute pg rows
    let mut stream = stream::iter(&prices);
    while let Some(cell) = stream.next().await {
        let query = query.clone();
        let tx = tx.clone();
        async move {
            tx.execute(
                &query,
                &[
                    &PLACEHOLDER_PK,
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
    Arc::into_inner(tx)
        .expect("failed to unpack Transaction from Arc")
        .commit()
        .await
        .unwrap();

    Ok(())
}

// -- COPY INTO POSTGRES --
async fn copy(prices: Vec<Price>, pg_client: &mut PgClient) -> anyhow::Result<()> {
    // Start a transaction
    let tx = pg_client.transaction().await?;

    // Prepare the COPY statement
    let copy_stmt = "
        COPY stock.prices (symbol_pk, dt, interval_pk, opening, high, low, closing, adj_close, volume) 
        FROM STDIN WITH (FORMAT binary)
    ";

    // Begin the COPY operation
    let sink = tx.copy_in(copy_stmt).await?;
    let writer = BinaryCopyInWriter::new(
        sink,
        &[
            Type::INT4,
            Type::TIMESTAMPTZ,
            Type::INT2,
            Type::FLOAT8,
            Type::FLOAT8,
            Type::FLOAT8,
            Type::FLOAT8,
            Type::FLOAT8,
            Type::INT8,
        ],
    );
    futures::pin_mut!(writer);

    // Iterate over the data stream and write to the sink in binary format
    let mut stream = stream::iter(&prices);
    while let Some(cell) = stream.next().await {
        writer
            .as_mut()
            .write(&[
                &PLACEHOLDER_PK as &(dyn ToSql + Sync),
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

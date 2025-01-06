use dotenv::var;
use std::collections::HashSet as Set;
use tokio_postgres::{self as pg, NoTls};

#[tokio::test]
async fn select_pks() {
    // -- CONNECT TO POSTGRES --
    let time = std::time::Instant::now();
    let (pg_client, pg_conn) = pg::connect(
        &var("FINDUMP_URL").expect("environment variable FINDUMP_URL"),
        NoTls,
    )
    .await
    .unwrap();
    println!("CONNECT: {:?}s", time.elapsed().as_secs_f64());

    tokio::spawn(async move {
        if let Err(err) = pg_conn.await {
            panic!("findump connection error: {}", err)
        }
    });

    // -- SELECT DISTINCT DATETIMES --
    let time = std::time::Instant::now();
    let pks: Set<chrono::DateTime<chrono::Utc>> = pg_client
        .query(
            "
            SELECT DISTINCT dt FROM stock.prices p 
            INNER JOIN stock.tickers t ON t.pk = p.symbol_pk
            WHERE t.ticker = $1
        ",
            &[&"MSFT"],
        )
        .await
        .unwrap()
        .iter()
        .map(|row| row.get(0))
        .collect();
    println!(
        "SELECT DISTINCT datetimes: {:?}s, count: {}",
        time.elapsed().as_secs_f64(),
        pks.len()
    );

    // -- SELECT ALL PRIMARY KEYS --
    let time = std::time::Instant::now();
    let rows = pg_client
        .query(
            "
            SELECT symbol_pk, dt, interval_pk FROM stock.prices p 
            INNER JOIN stock.tickers t ON t.pk = p.symbol_pk
            WHERE t.ticker = $1
        ",
            &[&"MSFT"],
        )
        .await
        .unwrap();

    let all: Set<PricePrimaryKey> = rows
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
        all.len()
    );
}

// -- DESERIALIZATION --
#[derive(Clone, Debug)]
struct Price {
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

#[derive(Debug, PartialEq, Eq, Hash)]
struct PricePrimaryKey {
    stock_pk: i32,
    time: chrono::DateTime<chrono::Utc>,
    interval_pk: i16,
}

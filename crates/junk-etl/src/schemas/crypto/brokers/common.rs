use anyhow::{anyhow, Result};
use deadpool_postgres::Pool;
use std::collections::HashMap;
use tracing::{debug, error};

/// Insert a row of price data.
pub(super) const INSERT_PRICE: &str = r#"
INSERT INTO crypto.fact_prices (
    symbol_pk,
    dt,
    interval_pk,
    open,
    high,
    low,
    close,
    volume,
    trades,
    source_pk
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
ON CONFLICT (symbol_pk, dt, interval_pk, source_pk)
DO NOTHING
"#;

/// Insert a cryptocurrency pair's symbol.
pub(super) const INSERT_SYMBOL: &str = r#"
INSERT INTO crypto.ref_symbols (symbol)
VALUES ($1)
ON CONFLICT (symbol)
DO NOTHING
"#;

/// Retrieve the already-existing Symbols from the database, with their respective Primary Keys.
pub(crate) async fn existing_symbols(pool: &Pool) -> Result<HashMap<String, i32>> {
    let client = pool.get().await.expect("Failed to get client from pool");

    let stmt = client
        .prepare(
            r#"
        SELECT symbol, symbol_pk
        FROM crypto.ref_symbols
        "#,
        )
        .await?;

    let rows = client.query(&stmt, &[]).await?;
    drop(client); // drop the client as quickly as possible

    let map = rows
        .into_iter()
        .map(|row| (row.get("symbol"), row.get("symbol_pk")))
        .collect::<HashMap<String, i32>>();

    Ok(map)
}

/// Retrieve the already-existing Sources from the database, with their respective Primary Keys.
pub(crate) async fn existing_source(pool: &Pool, source: String) -> Result<i16> {
    let client = pool.get().await.expect("Failed to get client from pool");

    // Attempt to find the Source PK in the existing table.
    let pk = match client
        .query_one(
            "SELECT source_pk FROM crypto.ref_sources WHERE source = $1",
            &[&source],
        )
        .await
    {
        Ok(pk) => pk,

        // If no PK is found, insert a new one, and reattempt to find it.
        Err(_e) => {
            debug!("{source} was not found in crypto.ref_symbols; inserting new source ...");
            client
                .query_one(
                    "INSERT INTO crypto.ref_symbols (source) VALUES ($1)",
                    &[&source],
                )
                .await?;
            match client
                .query_one(
                    "SELECT source_pk FROM crypto.ref_sources WHERE source = $1",
                    &[&source],
                )
                .await
            {
                Ok(new_pk) => new_pk,
                Err(e) => {
                    error!("{source} failed again - aborting: {e}");
                    return Err(anyhow!(e));
                }
            }
        }
    };

    drop(client);

    Ok(pk.get("source_pk"))
}

pub(crate) async fn existing_intervals(
    pool: &Pool,
    interval: String,
) -> Result<HashMap<String, i16>> {
    let client = pool.get().await.expect("Failed to get client from pool");

    // Attempt to find the Source PK in the existing table.
    let pk = match client
        .query_one(
            "SELECT interval_pk FROM common_ref_intervals WHERE interval = $1",
            &[&interval],
        )
        .await
    {
        Ok(pk) => pk,

        // If no PK is found, insert a new one, and reattempt to find it.
        Err(_e) => {
            debug!("{interval} was not found in common.ref_intervals; inserting new source ...");
            client
                .query_one(
                    "INSERT INTO common.ref_intervals (interval) VALUES ($1)",
                    &[&intervak],
                )
                .await?;
            match client
                .query_one(
                    "SELECT interval_pk FROM common.ref_intervals WHERE interval = $1",
                    &[&interval],
                )
                .await
            {
                Ok(new_pk) => new_pk,
                Err(e) => {
                    error!("{interval} failed again - aborting: {e}");
                    return Err(anyhow!(e));
                }
            }
        }
    };

    drop(client);

    Ok(pk.get("source_pk"))
}

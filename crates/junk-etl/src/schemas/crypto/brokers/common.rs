use anyhow::Result;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

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
pub(crate) async fn existing_symbols(pool: &PgPool) -> Result<HashMap<String, i32>> {
    let rows = sqlx::query(
        r#"
        SELECT symbol, symbol_pk
        FROM crypto.ref_symbols
        "#,
    )
    .fetch_all(pool)
    .await?;

    let map = rows
        .into_iter()
        .map(|row| (row.get("symbol"), row.get("symbol_pk")))
        .collect::<HashMap<String, i32>>();

    Ok(map)
}
    
/// Retrieve the already-existing Sources from the database, with their respective Primary Keys.
pub(crate) async fn existing_sources(pool: &PgPool) -> Result<HashMap<String, i16>> {
    let rows = sqlx::query(
        r#"
        SELECT source, source_pk
        FROM crypto.ref_sources
        "#,
    )
    .fetch_all(pool)
    .await?;

    let map = rows
        .into_iter()
        .map(|row| (row.get("source"), row.get("source_pk")))
        .collect::<HashMap<String, i16>>();

    Ok(map)
}
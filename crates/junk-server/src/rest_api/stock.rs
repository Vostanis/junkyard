use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use serde::{Deserialize, Serialize};

/// Stock symbol.
#[derive(Deserialize, Serialize, utoipa::ToSchema)]
struct StockSymbols {
    symbol: String,
    title: String,
    industry: String,
}

/// Symbols
///
/// ```json
/// [
///     {
///         "symbol": "AAPL",
///         "title": "Apple Inc.",
///         "industry": "Technology"
///     },
///     ...
/// ]
/// ```
#[utoipa::path(
    get,
    path = "/stock/symbols",
    responses(
        (
            status = 200, 
            description = "\
            List of all stocks, their ticker symbols, and their respective industries \
            (according to National Governments)
            ", 
            body = [StockSymbols], 
            content_type = "application/json", 
            example = json!([
                {
                    "symbol": "AAPL", 
                    "title": "Apple Inc.", 
                    "industry": "Technology"
                }
            ])
        )
    )
)]
#[get("/stock/symbols")]
async fn symbols(db_pool: web::Data<Pool>) -> impl Responder {
    // establish connection from pool
    let conn: Client = db_pool.get().await.expect("get connection from pool");

    // query the database
    let query = "
    SELECT
        symbol,
        title,
        industry
    FROM stock.symbols";
    let rows = match conn.query(query, &[]).await { 
        Ok(rows) => rows,
        Err(e) => {
            println!("{e}");
            return HttpResponse::InternalServerError().body("Query execution failed");
        }
    };

    let data: Vec<StockSymbols> = rows
        .iter()
        .map(|row| StockSymbols {
            symbol: row.get("symbol"),
            title: row.get("title"),
            industry: row.get("industry"),
        })
        .collect();

    HttpResponse::Ok().json(data)
}
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

/// Stock symbol
#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct StockSymbols {
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
///         "title": "APPLE INC.",
///         "industry": "Electronic Computers"
///     },
///     {
///         "symbol": "MSFT",
///         "title": "MICROSOFT CORP",
///         "industry": "Services-Prepackaged Software"
///     },
///     {
///         "symbol": "NVDA",
///         "title": "NVIDIA CORP",
///         "industry": "Semiconductors & Related Devices"
///     },
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
                    "title": "APPLE INC.",
                    "industry": "Electronic Computers"
                },
                {
                    "symbol": "MSFT",
                    "title": "MICROSOFT CORP",
                    "industry": "Services-Prepackaged Software"
                },
                {
                    "symbol": "NVDA",
                    "title": "NVIDIA CORP",
                    "industry": "Semiconductors & Related Devices"
                },
            ])
        )
    )
)]
#[get("/stock/symbols")]
pub async fn symbols(db_pool: web::Data<PgPool>) -> impl Responder {
    // query the database using SQLx
    let query = "
    SELECT
        symbol,
        title,
        industry
    FROM stock.symbols";

    let result = sqlx::query(query)
        .map(|row: sqlx::postgres::PgRow| StockSymbols {
            symbol: row.get("symbol"),
            title: row.get("title"),
            industry: row.get("industry"),
        })
        .fetch_all(db_pool.get_ref())
        .await;

    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Query execution failed")
        }
    }
}

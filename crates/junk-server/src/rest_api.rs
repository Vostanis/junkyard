use actix_web::{get, web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// List of all Stocks
///
/// ```json
/// [
///     {
///         "industry": "Technology",
///         "ticker": "AAPL",
///         "title": "Apple Inc."
///     },
///     ...
/// ]
/// ```
#[derive(Deserialize, Serialize, utoipa::ToSchema)]
struct StockSymbols {
    ticker: String,
    title: String,
    industry: String,
}

#[utoipa::path(
    get,
    path = "/stock/symbols",
    responses(
        (
            status = 200,
            description = "\
                List of all listed companies, their ticker symbols, and their respective industries (according to National Governments)\
            ", 
            body = [StockSymbols], 
            content_type = "application/json", 
            example = json!([
                {
                    "ticker": "AAPL", 
                    "title": "Apple Inc.", 
                    "industry": "Technology"
                }
            ])
        )
    )
)]
#[get("/stock/symbols")]
async fn stock_symbols(db_pool: web::Data<Pool>) -> impl Responder {
    // establish connection from pool
    let conn: Client = db_pool.get().await.expect("get connection from pool");

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
            ticker: row.get("symbol"),
            title: row.get("title"),
            industry: row.get("industry"),
        })
        .collect();

    HttpResponse::Ok().json(data)
}

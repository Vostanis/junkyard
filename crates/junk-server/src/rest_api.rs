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
struct StockIndex {
    ticker: String,
    title: String,
    industry: String,
}

#[utoipa::path(
    get,
    path = "/stock/index",
    responses(
        (
            status = 200, description = "List of all stocks, their ticker symbols, and their respective industries (according to National Governments)", 
            body = [StockIndex], content_type = "application/json", 
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
#[get("stock/index")]
async fn index(db_pool: web::Data<Pool>) -> impl Responder {
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

    let data: Vec<StockIndex> = rows
        .iter()
        .map(|row| StockIndex {
            ticker: row.get("symbol"),
            title: row.get("title"),
            industry: row.get("industry"),
        })
        .collect();

    HttpResponse::Ok().json(data)
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// /// List of all metrics per stock ticker symbol
// ///
// /// ```json
// /// [
// ///    {
// ///         "date": "2021-01-01",
// ///         "metric": "GrossProfit",
// ///         "value": 123456.00
// ///     },
// ///     // ...
// /// ]
// /// ```
// #[derive(Deserialize, Serialize, utoipa::ToSchema)]
// struct StockMetrics {
//     date: String,
//     metric: String,
//     value: f64,
// }
//
// #[utoipa::path(
//     get,
//     path = "/stock/metrics/{ticker}",
//     responses(
//         (
//             status = 200, description = "Financial metrics of US stocks",
//             body = [StockMetrics], content_type = "application/json", 
//             example = json!([
//                 {
//                     "date": "2021-01-01", 
//                     "metric": "GrossProfit", 
//                     "value": 123456.00
//                 }
//             ])
//         )
//     ),
//     params(
//         ("ticker", description = "Stock ticker symbol")
//     )
// )]
// #[get("stock/metrics/{ticker}")]
// async fn metrics(path: web::Path<String>, db_pool: web::Data<Pool>) -> impl Responder {
//     // establish connection from pool
//     let conn: Client = db_pool.get().await.expect("get connection from pool");
//
//     // query the database
//     let ticker = path.into_inner();
//     let query = "
//     SELECT
//         dated::VARCHAR,
//         metric,
//         val
//     FROM q.stock_metrics WHERE ticker = $1";
//     let rows = match conn.query(query, &[&ticker]).await {
//         Ok(rows) => rows,
//         Err(e) => {
//             println!("{e}");
//             return HttpResponse::InternalServerError().body("Query execution failed");
//         }
//     };
//
//     let data: Vec<StockMetrics> = rows
//         .iter()
//         .map(|row| StockMetrics {
//             date: row.get("dated"),
//             metric: row.get("metric"),
//             value: row.get("val"),
//         })
//         .collect();
//
//     HttpResponse::Ok().json(data)
// }
//
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// /// List of all prices per stock ticker symbol
// ///
// /// ```json
// /// [
// ///    {
// ///         "time": "15127651235",
// ///         "interval": "1d",
// ///         "open": 123456.00,
// ///         "high": 123457.00,
// ///         "low": 123455.00,
// ///         "close": 123456.00,
// ///         "adj_close": 123456.00,
// ///         "volume": 999
// ///     },
// ///     // ...
// /// ]
// /// ```
// #[derive(Deserialize, Serialize, utoipa::ToSchema)]
// struct StockPrices {
//     time: i64,
//     interval: String,
//     open: f64,
//     high: f64,
//     low: f64,
//     close: f64,
//     adj_close: f64,
//     volume: u64,
// }
//
// #[utoipa::path(
//     get,
//     path = "/stock/prices/{ticker}",
//     responses(
//         (
//             status = 200, description = "Prices of US stocks", 
//             body = [StockPrices], content_type = "application/json", 
//             example = json!([
//                 {
//                     "time": 1500000000,
//                     "interval": "1d", 
//                     "open": 123456.00,
//                     "high": 123457.00,
//                     "low": 123455.00,
//                     "close": 123456.00,
//                     "adj_close": 123456.00,
//                     "volume": 999
//                 }
//             ])
//         )
//     ),
//     params(
//         ("ticker", description = "Stock ticker symbol")
//     )
// )]
// #[get("stock/prices/{ticker}")]
// async fn prices(path: web::Path<String>, db_pool: web::Data<Pool>) -> impl Responder {
//     // establish connection from pool
//     let conn: Client = db_pool.get().await.expect("get connection from pool");
//
//     // query the database
//     let ticker = path.into_inner();
//     let interval = "1d";
//     let query = "
//     SELECT
//         dated::VARCHAR,
//         metric,
//         val
//     FROM q.stock_prices WHERE ticker = $1 and interval = $2";
//     let rows = match conn.query(query, &[&ticker, &interval]).await {
//         Ok(rows) => rows,
//         Err(e) => {
//             println!("{e}");
//             return HttpResponse::InternalServerError().body("Query execution failed");
//         }
//     };
//
//     let data: Vec<StockMetrics> = rows
//         .iter()
//         .map(|row| StockMetrics {
//             date: row.get("dated"),
//             metric: row.get("metric"),
//             value: row.get("val"),
//         })
//         .collect();
//
//     HttpResponse::Ok().json(data)
// }

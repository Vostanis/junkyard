use actix_web::{get, web, HttpResponse, Responder};
use bigdecimal::BigDecimal;
use tera::{Context, Tera};

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
struct Ticker {
    pk: i32,
    symbol: String,
    title: String,
    industry: String,
}

/// Backend for the home page.
///
/// All dataset symbols are loaded in to allow for searching.
#[get("/home")]
pub async fn home(pool: web::Data<sqlx::PgPool>, tera: web::Data<Tera>) -> impl Responder {
    match sqlx::query_as::<_, Ticker>(r#"
    SELECT
        pk,
        symbol, 
        REGEXP_REPLACE(title, '[''\\\/]', '', 'g') AS title, 
        REGEXP_REPLACE(industry, '[''\\\/]', '', 'g') AS industry
    FROM stock.symbols"#)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(tickers) => {
            let mut context = Context::new();
            let tickers_json = serde_json::to_string(&tickers).expect("Failed to serialize tickers");
            context.insert("tickers", &tickers_json);
            let rendered = tera
                .render("home.html", &context)
                .expect("failed to render home");
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch stock symbol"),
    }
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Price {
    pub date: chrono::NaiveDate,
    pub perc: Option<f64>,
    pub adj_close: f64,
    pub adj_close_20ma: f64,
    pub adj_close_50ma: f64,
    pub adj_close_200ma: f64,
    pub volume: i64,
    pub volume_7ma: BigDecimal,
    pub volume_90ma: BigDecimal,
}

/// Backend for an individual stock's dashboard.
#[get("/asset/{symbol}")]
pub async fn stock_dashboard(
    symbol: web::Path<String>,
    pool: web::Data<sqlx::PgPool>,
    tera: web::Data<Tera>,
) -> impl Responder {
    let symbol = symbol.into_inner();

    match sqlx::query_as::<_, Price>(
        "
        SELECT dt::DATE AS date, perc, adj_close, adj_close_20ma, adj_close_50ma, adj_close_200ma, volume, volume_7ma, volume_90ma
        FROM stock.prices_matv  
        WHERE symbol = $1
        ORDER BY date DESC
        -- LIMIT 250
    ",
    )
    .bind(&symbol)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(prices) => {
            let mut context = Context::new();
            let prices_json = serde_json::to_string(&prices).expect("failed to serialize prices");
            context.insert("prices", &prices_json);
            let rendered = tera
                .render("stock_dashboard.html", &context)
                .expect("failed to render stock_dashboard");
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
        Err(e) => {
            tracing::error!("{e}");
            HttpResponse::InternalServerError().body("Failed to fetch stock data")
        }
    }
}

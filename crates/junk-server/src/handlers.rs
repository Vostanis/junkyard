use actix_web::{get, web, HttpResponse, Responder};
use bigdecimal::BigDecimal;
use sqlx::Row;
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
    match sqlx::query_as::<_, Ticker>(
        r#"
    SELECT
        pk,
        symbol, 
        REGEXP_REPLACE(title, '[''\\\/]', '', 'g') AS title, 
        REGEXP_REPLACE(industry, '[''\\\/]', '', 'g') AS industry
    FROM stock.symbols"#,
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(tickers) => {
            let mut context = Context::new();
            let tickers_json =
                serde_json::to_string(&tickers).expect("Failed to serialize tickers");
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

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct StdFins {
    pub end_date: chrono::NaiveDate,
    pub price: Option<f64>,
    pub shares_outstanding: Option<f64>,
    pub market_cap: Option<f64>,
    pub revenue: Option<f64>,
    pub gross_profit: Option<f64>,
    pub operating_income: Option<f64>,
    pub earnings: Option<f64>,
    pub earnings_perc: Option<f64>,
    pub avg_shares: Option<f64>,
    pub eps: Option<f64>,
    pub accumulated_earnings: Option<f64>,
    pub debt: Option<f64>,
    pub equity: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub assets: Option<f64>,
    pub float: Option<f64>,
    pub value_of_shares_bought_back: Option<f64>,
    pub dividend_payout: Option<f64>,
}

#[get("/asset/{symbol}")]
pub async fn stock_dashboard(
    symbol: web::Path<String>,
    pool: web::Data<sqlx::PgPool>,
    tera: web::Data<Tera>,
) -> impl Responder {
    let symbol = symbol.into_inner();

    match sqlx::query(
        r#"
        SELECT 
            json_build_object(
                'symbols', (
                    SELECT json_agg(
                        json_build_object(
                            'symbol', symbol,
                            'title', REGEXP_REPLACE(title, '[''\\\/]', '', 'g'),
                            'industry', REGEXP_REPLACE(industry, '[''\\\/]', '', 'g')  
                        )
                    )
                    FROM stock.symbols
                ),
                'prices', (
                    SELECT json_agg(
                        json_build_object(
                            'date', dt::DATE,
                            'perc', perc,
                            'adj_close', adj_close,
                            'adj_close_20ma', adj_close_20ma,
                            'adj_close_50ma', adj_close_50ma,
                            'adj_close_200ma', adj_close_200ma,
                            'volume', volume,
                            'volume_7ma', volume_7ma,
                            'volume_90ma', volume_90ma
                        )
                        ORDER BY dt::DATE DESC
                    )
                    FROM stock.prices_matv
                    WHERE symbol = $1
                ),
                'financials', (
                    SELECT json_agg(
                        json_build_object(
                            'end_date', end_date,
                            'price', price,
                            'revenue', revenue,
                            'earnings', earnings,
                            'earnings_perc', earnings_perc,
                            'eps', eps,
                            'gross_profit', gross_profit,
                            'operating_income', operating_income,
                            'accumulated_earnings', accumulated_earnings,
                            'debt', debt,
                            'equity', equity,
                            'return_on_equity', return_on_equity,
                            'debt_to_equity', debt_to_equity,
                            'assets', assets,
                            'return_on_assets', return_on_assets,
                            'market_cap', market_cap,
                            'shares_outstanding', shares_outstanding,
                            'float', float,
                            'value_of_shares_bought_back', value_of_shares_bought_back
                        )
                        ORDER BY end_date DESC
                    )
                    FROM stock.std_financials std
                    INNER JOIN stock.symbols sy ON sy.pk = std.symbol_pk
                    WHERE sy.symbol = $1
                )
            ) as combined_data
        "#,
    )
    .bind(&symbol)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => {
            let combined_data: serde_json::Value = row.get("combined_data");

            let mut context = Context::new();

            let symbols_json = serde_json::to_string(&combined_data["symbols"])
                .expect("Failed to serialize symbols to JSON");
            context.insert("symbols", &symbols_json);

            let prices_json = serde_json::to_string(&combined_data["prices"])
                .expect("Failed to serialize prices to JSON");
            context.insert("prices", &prices_json);

            let financials_json = serde_json::to_string(&combined_data["financials"])
                .expect("Failed to serialize prices to JSON");
            context.insert("financials", &financials_json);

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

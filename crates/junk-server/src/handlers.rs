use actix_web::{get, web, HttpResponse, Responder};
use tera::{Context, Tera};

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
struct Ticker {
    pk: i32,
    symbol: String,
    title: String,
    industry: String,
}

#[get("/home")]
pub async fn home(pool: web::Data<sqlx::PgPool>, tera: web::Data<Tera>) -> impl Responder {
    match sqlx::query_as::<_, Ticker>("SELECT pk, symbol, title, industry FROM stock.symbols")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(tickers) => {
            let mut context = Context::new();
            context.insert("tickers", &tickers);
            let rendered = tera
                .render("home.html", &context)
                .expect("failed to render home");
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch stock symbol"),
    }
}

#[get("/stock/{asset}")]
pub async fn stock_dashboard(
    pool: web::Data<sqlx::PgPool>,
    tera: web::Data<Tera>,
) -> impl Responder {
    match sqlx::query_as::<_, Ticker>("SELECT pk, symbol, title, industry FROM stock.symbols")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(tickers) => {
            let mut context = Context::new();
            context.insert("tickers", &tickers);
            let rendered = tera
                .render("home.html", &context)
                .expect("failed to render home");
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch stock symbol"),
    }
}

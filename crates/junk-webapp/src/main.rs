use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::var;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

mod handlers;
mod rest_api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = tera::Tera::new("templates/**/*").unwrap();
    // std::env::set_var("RUST_LOG", "actix_web=debug");
    dotenv::dotenv().ok();
    env_logger::init();

    // build pool from .env DATABASE_URL
    let db_url = var("FINDUMP_URL").expect("FINDUMP_URL must be set");
    let pool = sqlx::PgPool::connect(&db_url)
        .await
        .expect("failed to connect to findump");

    // create API documentation
    #[derive(OpenApi)]
    #[openapi(paths(rest_api::index))]
    struct ApiDoc;

    // run server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            .service(rest_api::index)
            .service(handlers::login)
            .service(handlers::home)
            .service(handlers::stock_dashboard)
            .service(Redoc::with_url("/redoc", ApiDoc::openapi()))
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 11234))?
    .run()
    .await
}

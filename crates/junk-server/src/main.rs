mod rest_api;

use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use dotenv::{dotenv, var};
use tokio_postgres::NoTls;
use actix_web::{web, middleware::Logger, App, HttpServer};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    dotenv::dotenv().ok();
    // env_logger::init();

    // build pool from .env DATABASE_URL
    let db_url = var("FINDUMP_URL").expect("FINDUMP_URL must be set");
    let mut cfg = Config::new();
    cfg.url = Some(db_url);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool");

    // create API documentation
    #[derive(OpenApi)]
    #[openapi(paths(rest_api::index))]
    struct ApiDoc;
    let openapi = ApiDoc::openapi();

    // run server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(rest_api::index)
            .service(Redoc::with_url("/redoc", ApiDoc::openapi()))
    })
    .bind(("127.0.0.1", 11234))?
    .run()
    .await
}

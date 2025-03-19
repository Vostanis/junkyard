use actix_files as fs;
use actix_web::{web, App, HttpServer};
use dotenv::{dotenv, var};
use log::info;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

mod rest_api;
use rest_api::stock;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // build pool from .env DATABASE_URL
    let db_url = var("FINDUMP_URL").expect("FINDUMP_URL must be set");
    let pool = web::Data::new(
        sqlx::PgPool::connect(&db_url)
            .await
            .expect("failed to connect to findump"),
    );

    let bind_address = "127.0.0.1:8081";
    info!("Starting server at http://{}", bind_address);

    // create API documentation
    #[derive(OpenApi)]
    #[openapi(paths(stock::symbols))]
    struct ApiDoc;

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            /* API endpoints */
            // 1. stock schema
            .service(stock::symbols)
            // .service(stock::prices)
            // .service(stock::metrics)
            // .service(stock::aggregates)
            // 2. crypto schema
            // Serve the REST API Documentation
            .service(Redoc::with_url("/redoc", ApiDoc::openapi()))
            // Serve the WASM package files
            .service(fs::Files::new("/pkg", "./static/pkg").show_files_listing())
            // Serve static files and set index.html as the default
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(bind_address)?
    .run()
    .await
}

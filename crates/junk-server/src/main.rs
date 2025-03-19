use actix_files as fs;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use log::info;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

mod rest_api;
use rest_api::*;

// create API documentation
#[derive(OpenApi)]
// #[openapi(paths(rest_api::stock_symbols))]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let bind_address = "127.0.0.1:8081";
    info!("Starting server at http://{}", bind_address);

    HttpServer::new(|| {
        App::new()
            // API endpoints
            .service(hello_api)
            .service(health_check)
            .service(stock::symbols)
            .service(stock::prices)
            .service(stock::metrics)
            .service(stock::aggregates)
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

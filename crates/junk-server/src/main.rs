use actix_files as fs;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

mod rest_api;

// create API documentation
#[derive(OpenApi)]
#[openapi(paths(rest_api::stock_symbols))]
struct ApiDoc;

// Simple API response structure
#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
}

// API endpoint to test server-client communication
#[get("/api/hello")]
async fn hello_api() -> impl Responder {
    let response = ApiResponse {
        message: "Hello from Actix-Web API!".to_string(),
        status: "success".to_string(),
    };

    HttpResponse::Ok().json(response)
}

// Health check endpoint
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Server is running!")
}

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
            .service(rest_api::stock_symbols)
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

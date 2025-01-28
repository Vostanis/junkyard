use actix_web::{web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path) -> impl Responder {
    format!("hello {}", name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(greet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

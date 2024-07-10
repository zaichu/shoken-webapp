use actix_web::{web, App, HttpServer, Responder};
use std::env;

async fn greet() -> impl Responder {
    "Hello, world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    HttpServer::new(|| App::new().route("/", web::get().to(greet)))
        .bind(addr)?
        .run()
        .await
}

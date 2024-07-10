use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct Hoge {
    hoge: String,
    piyo: u8,
}

#[get("/json")]
async fn json() -> impl Responder {
    let hoge = Hoge {
        hoge: "hoge piyo".to_string(),
        piyo: 8,
    };
    HttpResponse::Ok().json(hoge)
}

#[get("/")]
async fn index() -> impl Responder {
    "こんちは。"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    HttpServer::new(|| App::new().service(index).service(json))
        .bind(addr)?
        .run()
        .await
}

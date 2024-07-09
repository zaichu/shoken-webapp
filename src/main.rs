use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

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
    HttpServer::new(|| App::new().service(index).service(json))
        .bind("localhost:8080")?
        .run()
        .await
}

use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpServer, Result};
use futures_util::TryStreamExt;
use std::env;

mod services;
use services::csv_processor;

async fn index() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("asset/html/index.html")?)
}

async fn process_csv(mut payload: Multipart, path: web::Path<String>) -> Result<String, Error> {
    let csv_type = path.into_inner();
    let mut field = payload.try_next().await?.unwrap();
    let result = csv_processor::process_csv(&mut field, &csv_type)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting server at: {addr}");

    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/css", "asset/css").show_files_listing())
            .route("/", web::get().to(index))
            .route("/process-csv/{type}", web::post().to(process_csv))
    })
    .bind(addr)?
    .run()
    .await
}

mod config;
use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpServer, Result};
use futures_util::TryStreamExt;

mod services;
use services::csv_processor;

async fn process_csv(mut payload: Multipart, path: web::Path<String>) -> Result<String, Error> {
    let csv_type = path.into_inner();
    let mut field = match payload.try_next().await {
        Ok(Some(field)) => field,
        Ok(None) => return Err(actix_web::error::ErrorBadRequest("No file in payload")),
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    };

    let result = csv_processor::process_csv(&mut field, &csv_type)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::get_config();
    println!("Starting server at: {}", config.addr);

    HttpServer::new(move || {
        App::new()
            .service(fs::Files::new("/js", "asset/js").show_files_listing())
            .service(fs::Files::new("/css", "asset/css").show_files_listing())
            .service(fs::Files::new("/img", "asset/img").show_files_listing())
            .route("/process-csv/{type}", web::post().to(process_csv))
            .service(fs::Files::new("/", "asset/html").index_file("index.html"))
    })
    .bind(config.addr)?
    .run()
    .await
}

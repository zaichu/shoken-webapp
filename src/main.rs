mod setting;
use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Result};
use futures::StreamExt;
use futures_util::TryStreamExt;
use std::{env, sync::Arc};
use tera::Tera;

mod services;
use services::factroy;

async fn process_csv(mut payload: Multipart, path: web::Path<String>) -> Result<String, Error> {
    let csv_type = path.into_inner();
    let mut field = match payload.try_next().await {
        Ok(Some(field)) => field,
        Ok(None) => return Err(actix_web::error::ErrorBadRequest("No file in payload")),
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    };

    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk.map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
        bytes.extend_from_slice(&chunk);
    }

    let manager = factroy::create_factory(&csv_type);
    let result = manager
        .execute(bytes)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(result)
}

async fn index(tmpl: web::Data<Arc<Tera>>) -> Result<HttpResponse, Error> {
    let context = tera::Context::new();
    let rendered = tmpl
        .render("index.html", &context)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

async fn receipts(tmpl: web::Data<Arc<Tera>>) -> Result<HttpResponse, Error> {
    let context = tera::Context::new();
    let rendered = tmpl
        .render("receipts.html", &context)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting server at: {}", addr);
    let tera = Arc::new(Tera::new("asset/html/**").unwrap());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .service(fs::Files::new("/js", "asset/js").show_files_listing())
            .service(fs::Files::new("/css", "asset/css").show_files_listing())
            .service(fs::Files::new("/img", "asset/img").show_files_listing())
            .route("/process-csv/{type}", web::post().to(process_csv))
            .route("/", web::get().to(index))
            .route("/receipts", web::get().to(receipts))
    })
    .bind(addr)?
    .run()
    .await
}

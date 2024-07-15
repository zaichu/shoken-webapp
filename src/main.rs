mod config;
mod handlers;
mod services;

use actix_files as fs;
use actix_web::{App, HttpServer};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let config = config::get_config();
    println!("Starting server at: {}", config.addr);

    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/js", "asset/js").show_files_listing())
            .service(fs::Files::new("/img", "asset/img").show_files_listing())
            .service(handlers::upload::upload)
            .service(fs::Files::new("/", "asset/html").index_file("index.html"))
    })
    .bind(config.addr)?
    .run()
    .await
}

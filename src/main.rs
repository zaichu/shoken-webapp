use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use encoding_rs::SHIFT_JIS;
use futures_util::stream::StreamExt as _;
use std::env;
use std::io;

#[actix_web::get("/")]
async fn index() -> impl Responder {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Upload CSV</title>
        </head>
        <body>
            <h1>Upload CSV</h1>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <input type="file" name="file" />
                <button type="submit">Upload</button>
            </form>
        </body>
        </html>
    "#;
    HttpResponse::Ok().body(html)
}

#[actix_web::post("/upload")]
async fn upload(mut payload: Multipart) -> impl Responder {
    let mut table = String::from("<table border=\"1\">");

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().unwrap();
        let filename = content_disposition.get_filename().unwrap();

        if filename.ends_with(".csv") {
            let mut bytes = web::BytesMut::new();

            while let Some(Ok(chunk)) = field.next().await {
                bytes.extend_from_slice(&chunk);
            }

            // Decode Shift JIS to UTF-8
            let (decoded, _, _) = SHIFT_JIS.decode(&bytes);
            let utf8_bytes = decoded.as_bytes();

            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(utf8_bytes);

            // Parse each record manually
            for result in rdr.records() {
                match result {
                    Ok(record) => {
                        table.push_str("<tr>");
                        for field in record.iter() {
                            table.push_str(&format!("<td>{}</td>", field));
                        }
                        table.push_str("</tr>");
                    }
                    Err(err) => {
                        eprintln!("Error parsing record: {:?}", err);
                        return HttpResponse::InternalServerError()
                            .body(format!("Error parsing record: {:?}", err));
                    }
                }
            }

            table.push_str("</table>");
            break;
        }
    }

    HttpResponse::Ok().body(table)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    HttpServer::new(|| App::new().service(index).service(upload))
        .bind(addr)?
        .run()
        .await
}

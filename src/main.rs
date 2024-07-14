use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chardetng::EncodingDetector;
use futures_util::stream::StreamExt as _;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{env, io};

#[actix_web::post("/upload")]
async fn upload(mut payload: Multipart) -> impl Responder {
    let mut table = String::from("<html lang=\"ja\"><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"></head><table border=\"1\">");

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().unwrap();
        let filename = content_disposition.get_filename().unwrap();

        if filename.ends_with(".csv") {
            let mut bytes = web::BytesMut::new();

            while let Some(Ok(chunk)) = field.next().await {
                bytes.extend_from_slice(&chunk);
            }

            let mut detector = EncodingDetector::new();
            detector.feed(&bytes, true);
            let encoding = detector.guess(None, true);
            let (decoded, _, _) = encoding.decode(&bytes);
            let utf8_string = decoded.into_owned();

            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(utf8_string.as_bytes());

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

            table.push_str("</table></html>");
            break;
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(table)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder
    //     .set_private_key_file("key.pem", SslFiletype::PEM)
    //     .unwrap();
    // builder.set_certificate_chain_file("cert.pem").unwrap();

    // 先に追加したserviceからマッチするため、"/"を一番最初に書くと「/js」「/img」がマッチされなくなってしまう
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/js", "asset/js").show_files_listing())
            .service(fs::Files::new("/img", "asset/img").show_files_listing())
            .service(upload)
            // .service(fs::Files::new("/", "asset/html").show_files_listing())
            .service(fs::Files::new("/", "asset/html").index_file("index.html"))
    })
    // .bind_openssl(addr, builder)?
    .bind(addr)?
    .run()
    .await
}

use crate::services::csv_processor;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder};
use futures_util::StreamExt;

#[actix_web::post("/upload")]
pub async fn upload(mut payload: Multipart) -> impl Responder {
    let mut response = String::from("<html lang=\"ja\"><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"></head>");

    while let Some(field_result) = payload.next().await {
        let mut field = match field_result {
            Ok(field) => field,
            Err(e) => {
                eprintln!("Error processing multipart: {:?}", e);
                return HttpResponse::InternalServerError().body("Error processing upload");
            }
        };

        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => return HttpResponse::BadRequest().body("Content-Disposition not found"),
        };

        let filename = match content_disposition.get_filename() {
            Some(name) => name,
            None => return HttpResponse::BadRequest().body("Filename not found"),
        };

        if filename.ends_with(".csv") {
            match csv_processor::process_csv(&mut field).await {
                Ok(table) => response.push_str(&table),
                Err(e) => {
                    eprintln!("Error processing CSV: {:?}", e);
                    return HttpResponse::InternalServerError().body("Error processing CSV");
                }
            }
            break;
        }
    }

    response.push_str("</html>");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(response)
}

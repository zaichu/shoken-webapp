use crate::models::common::MessageResponse;
use axum::{http::StatusCode, response::Json};

pub fn ok_message(message: &str) -> (StatusCode, Json<MessageResponse>) {
    (
        StatusCode::OK,
        Json(MessageResponse {
            message: message.to_string(),
        }),
    )
}

use gloo::utils::format::JsValueSerdeExt;
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::{data::stock::StockData, env, setting::*};

#[derive(Debug, Error, PartialEq)]
pub enum ApiError {
    #[error("Request failed")]
    RequestError,
    #[error("No window object")]
    NoWindowObject,
    #[error("Fetch error")]
    FetchError,
    #[error("Response error")]
    ResponseError,
    #[error("JSON error")]
    JsonError,
    #[error("Deserialization error")]
    DeserializationError,
}

impl ApiError {
    #[allow(dead_code)]
    fn from_js_error<E: Into<ApiError>>(error: E) -> ApiError {
        error.into()
    }
}

async fn fetch_json(url: &str) -> Result<JsValue, ApiError> {
    let window = web_sys::window().ok_or(ApiError::NoWindowObject)?;
    let request = create_request(url)?;

    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| ApiError::FetchError)?;
    let response: Response = response.dyn_into().map_err(|_| ApiError::ResponseError)?;

    JsFuture::from(response.json().map_err(|_| ApiError::JsonError)?)
        .await
        .map_err(|_| ApiError::JsonError)
}

fn create_request(url: &str) -> Result<Request, ApiError> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    Request::new_with_str_and_init(url, &opts).map_err(|_| ApiError::RequestError)
}

pub async fn fetch_stock_data(code: &str) -> Result<StockData, ApiError> {
    let url = format!("{}/stock/{}", env::SHOKEN_WEBAPI_API_URL, code);
    let json = fetch_json(&url).await?;

    json.into_serde::<StockData>()
        .map_err(|_| ApiError::DeserializationError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let error = ApiError::RequestError;
        assert_eq!(error.to_string(), "Request failed");

        let error = ApiError::NoWindowObject;
        assert_eq!(error.to_string(), "No window object");

        let error = ApiError::FetchError;
        assert_eq!(error.to_string(), "Fetch error");

        let error = ApiError::ResponseError;
        assert_eq!(error.to_string(), "Response error");

        let error = ApiError::JsonError;
        assert_eq!(error.to_string(), "JSON error");

        let error = ApiError::DeserializationError;
        assert_eq!(error.to_string(), "Deserialization error");
    }

    #[test]
    fn test_from_js_error() {
        let error = ApiError::from_js_error(ApiError::RequestError);
        assert_eq!(error, ApiError::RequestError);
    }

    #[allow(dead_code)]
    fn test_create_request() {
        let url = "https://example.com/api";
        let result = create_request(url);

        assert!(result.is_ok());
    }
}

use crate::errors::ApiError;
use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::Request,
    Json,
};
use serde::de::DeserializeOwned;
use tracing::error;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let result: Result<Json<T>, JsonRejection> = Json::<T>::from_request(req, state).await;
        let Json(value) = result.map_err(|e| {
            error!("[ValidatedJson] JSONパースエラー: {}", e);
            ApiError::JsonParseError
        })?;

        value.validate().map_err(|rejection| {
            let msg = format!("{}", rejection).replace('\n', ", ");
            error!("[ValidatedJson] バリデーションエラー: {}", msg);
            ApiError::ValidationError(msg)
        })?;

        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[rustfmt::skip]
    use axum::{body::Body, extract::FromRequest, http::{Request, StatusCode}};
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;
    use validator::Validate;

    #[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
    struct TestData {
        #[validate(length(min = 1, max = 50))]
        name: String,
        #[validate(range(min = 1, max = 150))]
        age: u8,
    }

    #[rustfmt::skip]
    fn json_request(body: impl Into<Body>) -> Request<Body> { Request::builder().header("content-type", "application/json").method("POST").uri("/test").body(body.into()).unwrap() }

    #[tokio::test]
    #[rustfmt::skip]
    async fn test_valid_json() {
        let json_data = TestData { name: "テストユーザー".to_string(), age: 30 };
        let app = tower::service_fn(|req: Request<Body>| async { let ValidatedJson(data) = ValidatedJson::<TestData>::from_request(req, &()).await.unwrap(); assert_eq!(data, json_data); Ok::<_, hyper::Error>(axum::response::Response::new(Body::empty())) });
        assert_eq!(app.oneshot(json_request(serde_json::to_string(&json_data).unwrap())).await.unwrap().status(), StatusCode::OK);
        assert!(matches!(ValidatedJson::<TestData>::from_request(json_request(r#"{"name": "テストユーザー", age: 30}"#), &()).await.unwrap_err(), ApiError::JsonParseError));
        assert!(matches!(ValidatedJson::<TestData>::from_request(json_request(serde_json::to_string(&TestData { name: "".to_string(), age: 30 }).unwrap()), &()).await.unwrap_err(), ApiError::ValidationError(_)));
    }
}

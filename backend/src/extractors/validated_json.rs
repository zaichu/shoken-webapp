use crate::errors::ApiError;
use axum::{extract::FromRequest, http::Request, Json};
use serde::de::DeserializeOwned;
use tracing::error;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| {
                error!("[ValidatedJson] JSONパースエラー");
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
    use axum::{
        body::Body,
        extract::FromRequest,
        http::{Request, StatusCode},
    };
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;
    use validator::Validate;

    #[derive(Debug, Serialize, Deserialize, Validate, PartialEq)]
    struct TestData {
        #[validate(length(min = 1, max = 50))]
        name: String,
        #[validate(range(min = 1, max = 150))]
        age: u8,
    }

    #[tokio::test]
    async fn test_valid_json() {
        let json_data = TestData {
            name: "テストユーザー".to_string(),
            age: 30,
        };
        let body = Body::from(serde_json::to_string(&json_data).unwrap());
        let request = Request::builder()
            .header("content-type", "application/json")
            .method("POST")
            .uri("/test")
            .body(body)
            .unwrap();

        let app = tower::service_fn(|req: Request<Body>| async {
            let ValidatedJson(data) = ValidatedJson::<TestData>::from_request(req, &())
                .await
                .unwrap();
            assert_eq!(data, json_data);
            Ok::<_, hyper::Error>(axum::response::Response::new(Body::empty()))
        });

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_json_syntax() {
        let body = Body::from(r#"{"name": "テストユーザー", age: 30}"#);
        let request = Request::builder()
            .header("content-type", "application/json")
            .method("POST")
            .uri("/test")
            .body(body)
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;
        assert!(result.is_err());

        match result {
            Err(ApiError::JsonParseError) => (),
            _ => panic!("Expected JsonParseError"),
        }
    }

    #[tokio::test]
    async fn test_invalid_validation() {
        let json_data = TestData {
            name: "".to_string(),
            age: 30,
        };
        let body = Body::from(serde_json::to_string(&json_data).unwrap());
        let request = Request::builder()
            .header("content-type", "application/json")
            .method("POST")
            .uri("/test")
            .body(body)
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(request, &()).await;
        assert!(result.is_err());

        match result {
            Err(ApiError::ValidationError(_)) => (),
            _ => panic!("Expected ValidationError"),
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_response_omits_absent_details() {
        let response = ErrorResponse {
            error: ErrorDetails {
                code: "VALIDATION_ERROR".to_string(),
                message: "入力内容を確認してください".to_string(),
                details: None,
            },
        };

        let json = serde_json::to_value(&response).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "error": {
                    "code": "VALIDATION_ERROR",
                    "message": "入力内容を確認してください"
                }
            })
        );

        let parsed: ErrorResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed, response);
    }

    #[test]
    fn error_response_keeps_details_when_present() {
        let parsed: ErrorResponse = serde_json::from_str(
            r#"{"error": {"code": "DATABASE_ERROR", "message": "Database error occurred", "details": "connection failed"}}"#,
        )
        .expect("deserialize");
        assert_eq!(parsed.error.details.as_deref(), Some("connection failed"));
    }
}

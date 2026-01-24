---
name: backend-api
description: |
  Axum REST API エンドポイントの実装パターン。
  認証付きCRUD、バリデーション、エラーハンドリング。
  Use when: API追加、エンドポイント実装、ハンドラー作成を依頼された時。
---

# バックエンド API 実装

## ハンドラー構造

```rust
use axum::{extract::State, response::IntoResponse, Json};
use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    models::YourModel,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let items = sqlx::query_as!(
        YourModel,
        "SELECT * FROM your_table WHERE user_id = $1",
        auth_user.id()
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(items))
}
```

## ルート登録 (main.rs)

```rust
use axum::routing::{get, post, delete};

let app = Router::new()
    .route("/your-route", get(handlers::your::list))
    .route("/your-route/bulk", post(handlers::your::bulk_create))
    .route("/your-route/all", delete(handlers::your::delete_all));
```

## モデル定義

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct YourModel {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub user_id: Uuid,
    // 他のフィールド
}

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    // リクエストフィールド
}
```

## バルク作成（重複スキップ）

```rust
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<BulkCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut inserted = 0;
    for item in payload.items {
        let result = sqlx::query!(
            r#"
            INSERT INTO your_table (user_id, field1, field2)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            auth_user.id(),
            item.field1,
            item.field2
        )
        .execute(&state.pool)
        .await?;

        if result.rows_affected() > 0 {
            inserted += 1;
        }
    }

    Ok(Json(BulkCreateResponse {
        inserted,
        total: payload.items.len(),
    }))
}
```

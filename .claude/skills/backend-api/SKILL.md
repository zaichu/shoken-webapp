---
name: backend-api
description: |
  Axum REST API エンドポイントの実装パターン。
  認証付きCRUD、バリデーション、エラーハンドリング。
  Use when: API追加、エンドポイント実装、ハンドラー作成を依頼された時。
---

# バックエンド API 実装

実行環境は Cloudflare Workers のみ。DB アクセスは `crate::db`（tokio-postgres ベースの自前層。ホストは deadpool-postgres、wasm は Hyperdrive 経由）を使い、sqlx は使わない。

## ハンドラー構造

ハンドラーは HTTP 入出力に絞り、DB・ドメイン処理は `services/` 側に置く
（実例: `handlers/v1/dividends.rs` → `services/dividend.rs`）。

```rust
use axum::{extract::State, response::IntoResponse, Json};
use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::YourModel,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/your-resources",
    operation_id = "v1_your_resource_list",
    responses(
        (status = 200, description = "一覧を返す", body = Vec<YourModel>),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let items = your_service::list(&state.pool, auth_user.id()).await?;
    Ok(Json(items))
}
```

`#[utoipa::path]` から生成する `docs/openapi.json` が API 契約の正本。変更時は `bash scripts/check-openapi.sh` で同期を確認する。

## DB アクセス（services 層）

クエリは `crate::db::query*` に SQL と `vec![Bind::from(..)]` を渡して組み立て、`.fetch_*` / `.execute` に `&Db`（`state.pool`）を渡す。実例: `services/stock.rs`、`services/dividend.rs`。

```rust
use crate::db::{Bind, Db};
use crate::errors::ApiError;
use shared::value::UserId;

pub async fn list(pool: &Db, user_id: UserId) -> Result<Vec<YourModel>, ApiError> {
    let items = crate::db::query_as::<YourModel>(
        "SELECT id, user_id, field1 FROM your_table WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_all(pool)
    .await?;
    Ok(items)
}
```

- `query_as::<T>` は `T: crate::db::FromRow` の行を `fetch_all` / `fetch_one` / `fetch_optional` で読む
- `query` は `execute`（rows_affected）や `fetch_scalar` 系に使う
- 動的 SQL は `crate::db::QueryBuilder`（`services/domain/search.rs` 参照）
- トランザクションは `pool.begin()` → `Tx`（`services/auth.rs` 参照）
- `DbError` は `#[from]` で `ApiError::Database` に変換されるため `?` でよい

## ルート登録

```rust
use axum::{routing::get, Router};

pub fn your_resource_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/your-resources", get(handlers::v1::your_resources::list))
}
```

- ルートは `backend/src/handlers/v1.rs` の route composition（auth / data / stock_search / csv_upload のグループで対応するレート制限が決まる）に集約する。
- 外部公開 API は `/api/v1/<resource>` を標準にする。
- 一括置換は `PUT /api/v1/<collection>`、全削除は `DELETE /api/v1/<collection>` を使う。`/bulk` や `/all` を新規 API の標準例にしない。

## モデル定義

`FromRow` は derive ではなく手書き impl で実装する（実例: `models/stock.rs`）。

```rust
use serde::{Deserialize, Serialize};
use shared::value::UserId;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct YourModel {
    pub id: uuid::Uuid,
    #[serde(skip_serializing)]
    pub user_id: UserId,
    // 他のフィールド
}

impl crate::db::FromRow for YourModel {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            // 他のフィールド
        })
    }
}
```

`#[allow(dead_code)]` はテンプレートとして追加しない。DB の所有者 ID など、`FromRow` には必要だが Rust コードから直接読まないフィールドで clippy が警告する場合だけ、構造上の理由をコメントで明記して最小範囲に付与する。

## 一括登録（重複スキップ・置換）

`services/domain/bulk.rs` の `bulk_insert`（追記型・`ON CONFLICT DO NOTHING`）/ `bulk_replace`（置換型・全削除して INSERT）が共通骨格。ドメイン側は UNNEST の INSERT クエリを組み立てて渡すだけで、ユーザー行ロック・行数上限（`state.config.user_row_limit`）チェック・計測ログは共通側が持つ。

実例: `services/dividend.rs`（追記型）、`services/asset_balance.rs`（置換型）。UNNEST の型対応は `bulk-processing` スキルと `crate::db::Bind` の `From<Vec<..>>` impl を参照。

## J-Quants API V2 連携

### フィールド名の注意点

J-Quants API V2 は省略形フィールド名を使用。`serde(rename)` で正確に指定する必要がある。

| 項目 | API V2 フィールド名 |
|------|-------------------|
| 営業利益 | `OP` |
| 経常利益 | `OdP` |
| 当期純利益 | `NP` |
| 当期種別 | `CurPerType` |
| 当期開始日 | `CurPerSt` |
| 期末配当 | `DivFY` |
| 年間配当実績 | `DivAnn` |
| 年間配当予想 | `FDivAnn` |
| 年間配当来期予想 | `NxFDivAnn` |

### 型定義例

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct FinSummaryData {
    #[serde(rename = "DiscDate")]
    pub disclosed_date: String,

    #[serde(rename = "OP", default)]
    pub operating_profit: Option<String>,

    #[serde(rename = "NxFDivAnn", default)]
    pub next_year_forecast_dividend_per_share_annual: Option<String>,
}
```

### フロントエンドでの防御的コーディング

APIレスポンスの `data` フィールドが配列でない場合に備える：

```typescript
if (!response?.data || !Array.isArray(response.data)) {
  return;
}
for (const item of response.data) {
  // 処理
}
```

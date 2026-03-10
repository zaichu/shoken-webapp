# アーキテクチャ概要

## システム構成

```
┌─────────────────────────────────────────────────────────────┐
│ Browser                                                     │
│  React 19 + TypeScript (Vite)                               │
│  Vercel でホスティング                                        │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTPS / Axios
┌───────────────────────────▼─────────────────────────────────┐
│ Backend API (Fly.io)                                        │
│  Rust + Axum 0.8                                            │
│  ├── 認証ミドルウェア（CSRF / レート制限 / セキュリティヘッダー）│
│  ├── ハンドラー層 (handlers/)                                 │
│  ├── サービス層 (services/)                                   │
│  └── SQLx → PostgreSQL (Neon)                               │
└─────────────────────────────────────────────────────────────┘
         │                              │
         │ Google OAuth 2.0            │ J-Quants API
         ▼                              ▼
   accounts.google.com          jpx-jquants.com
```

## ディレクトリ構成

```
shoken-webapp/
├── frontend/
│   ├── src/
│   │   ├── components/    # Atomic Design (atoms/molecules/organisms/templates)
│   │   ├── features/      # auth / stock / receipt / assetBalance / jquants
│   │   ├── hooks/
│   │   ├── pages/
│   │   ├── lib/           # api client / csv utils / interfaces / types / utils
│   │   ├── contexts/
│   │   ├── routes/
│   │   └── styles/
│   └── e2e/               # Playwright E2E テスト
├── backend/
│   └── src/
│       ├── handlers/      # HTTP ハンドラー（ドメイン別）
│       ├── services/      # ビジネスロジック（CSV パース / DB アクセス）
│       ├── models/        # データモデル・バリデーション
│       ├── extractors/    # カスタム Axum エクストラクター
│       ├── middleware/      # rate_limit / security / tracing サブモジュール
│       ├── routes.rs      # ルーティング定義
│       ├── config.rs      # 環境変数読み込み
│       ├── errors.rs      # 統一エラーハンドリング
│       └── state.rs       # AppState (DB pool / secrets / HTTP client)
├── docs/                  # 設計・運用ドキュメント
└── .github/               # CI/CD ワークフロー / Dependabot 設定
```

## 認証フロー

```
1. フロント → GET /auth/google
2. バックエンド → Google OAuth 認証ページへリダイレクト
3. Google → GET /auth/google/callback?code=...&state=...
4. バックエンド → セッション Cookie 発行（session_token, Max-Age=7日）
5. フロント → GET /auth/me でユーザー情報取得
```

CSRF 対策: `Origin` / `Referer` ヘッダーによるオリジン検証

## セキュリティ層（ミドルウェアスタック）

内側から外側の順:

1. ルートハンドラー
2. ドメイン別ミドルウェア（keyed レート制限 / global レート制限）
3. `validate_origin`（CSRF 防止）
4. `RequestBodyLimitLayer`（10MB 上限）
5. CORS
6. `TraceLayer`（リクエストトレース、クエリパラメータ除外）
7. `PropagateRequestIdLayer`（x-request-id をレスポンスに伝播）
8. `SetRequestIdLayer`（x-request-id の UUID 自動付与）
9. `add_security_headers`（最外層: すべてのレスポンスに付与）

## CSV インポート設計

CSV ファイルのアップロードは2段階:

1. **プレビュー** `POST /xxx/csv/preview`: ファイルをパースして行エラー一覧を返す（DB 書き込みなし）
2. **確定保存** `POST /xxx/csv`: パース + DB 挿入（ON CONFLICT DO NOTHING / occurrence_index）

## データフロー（保有銘柄 CSV の場合）

```
Client → POST /asset-balances/csv
  → middleware stack
  → handler::asset_balance::upload_csv
  → services::csv_import::parse_csv (with parse_asset_balance_row)
  → services::asset_balance::bulk_create (DELETE ALL → INSERT)
  → 201 { inserted, skipped, errors }
```

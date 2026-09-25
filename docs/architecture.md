# アーキテクチャ概要

## 採用アーキテクチャ

- **Backend**: Modular Monolith with Thin HTTP Adapter Layer
  （Rust/Axum 単一バイナリ、ドメイン別モジュール構成、ハンドラーは薄い adapter）
- **Frontend**: Leptos (CSR) と Trunk でブラウザ向け WASM を配信

判断の根拠と「採用しないもの」は `docs/adr/0001-keep-axum-and-harden-api.md`
の Architecture Stance を正本とする。本ドキュメントは構成図と境界の早見表として
扱う。

## システム構成

```
┌─────────────────────────────────────────────────────────────┐
│ Browser                                                     │
│  Rust + Leptos (CSR)                                        │
│  Vercel でホスティング                                        │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTPS / fetch
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
├── frontend-leptos/
│   ├── src/               # Leptos UI / API client / domain logic
│   ├── e2e/               # Playwright E2E テストと CSV fixture
│   ├── style/             # Tailwind CSS
│   └── Trunk.toml
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

## 境界の置き方

「どこで線を引くか」を短く明文化する。詳細な判断は ADR 0001 を参照。

### Backend（Modular Monolith + Thin HTTP Adapter）

- **HTTP 境界**: `handlers/` と `extractors/` で完結させ、Axum 型はここから外に
  漏らさない。ハンドラーは「入力パース → service 呼び出し → エラーマップ」だけ。
- **サービス境界**: `services::<domain>` の関数がドメインの業務ルール・
  バリデーション・SQLx クエリを所有する。ハンドラー間でロジックを共有したい
  場合は HTTP ではなく service 関数を経由する。
- **永続化境界**: SQLx クエリは service モジュール内に置く。Repository trait
  は作らない。テストは実 DB（ローカル PostgreSQL）に対して書く。
- **共有状態**: `AppState`（DB pool / secrets / HTTP client）で配線する。DI
  container は導入しない。

### Frontend（Leptos）

- **API 境界**: `src/api/` が backend との通信を扱う。
- **UI とドメイン**: `src/` の画面モジュールとドメインモジュールが表示・計算を扱う。
- **契約**: `docs/openapi.json` を API 契約の正本とし、Leptos の契約テストで確認する。

### 引かない境界（採用しない）

- 全面ヘキサゴナル / 全面クリーンアーキテクチャ（ports & adapters の重複コスト
  に見合うサイズではない）
- 集約ごとの Repository trait（SQLx 直書きの方が変更が早い）
- DI container（`AppState` と関数引数で十分）
- backend の `domain / application / infrastructure` 三層分割

これらは具体的な痛み（第二の transport、サービス分割、DB fake 必須テスト等）が
発生してから再検討する。

## 認証フロー

```
1. フロント → GET /api/v1/oauth/google/authorize
2. バックエンド → Google OAuth 認証ページへリダイレクト
3. Google → GET /api/v1/oauth/google/callback?code=...&state=...
4. バックエンド → セッション Cookie 発行（session_token, Max-Age=7日）
5. フロント → GET /api/v1/session でユーザー情報取得
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

CSV ファイルのアップロードは2段階。詳細な API 契約は `docs/api/v1-rest-design.md` を参照。

1. **バリデーション** `POST /api/v1/{resource}-import-validations`: ファイルをパースして行エラー一覧を返す（DB 書き込みなし）
2. **確定インポート** `POST /api/v1/{resource}-imports`: パース + DB 挿入（ON CONFLICT DO NOTHING / occurrence_index）

エンドポイント例:

| リソース | バリデーション | インポート |
|---|---|---|
| 配当金 | `POST /api/v1/dividend-import-validations` | `POST /api/v1/dividend-imports` |
| 国内株式 | `POST /api/v1/domestic-stock-import-validations` | `POST /api/v1/domestic-stock-imports` |
| 投資信託 | `POST /api/v1/mutual-fund-import-validations` | `POST /api/v1/mutual-fund-imports` |
| 保有銘柄 | `POST /api/v1/asset-balance-import-validations` | `POST /api/v1/asset-balance-imports` |

## データフロー（保有銘柄 CSV の場合）

```
Client → POST /api/v1/asset-balance-import-validations  # バリデーション（DB 書き込みなし）
  → middleware stack
  → handlers::v1::asset_balances::validate_import
  → services::csv_import::parse_csv (with parse_asset_balance_row)
  → 200 { errors: [...] }

Client → POST /api/v1/asset-balance-imports             # 確定インポート
  → middleware stack
  → handlers::v1::asset_balances::import
  → services::csv_import::parse_csv (with parse_asset_balance_row)
  → services::asset_balance::bulk_create (DELETE ALL → INSERT)
  → 201 { inserted, skipped, errors }
```

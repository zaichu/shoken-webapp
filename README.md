# 証券情報ウェブアプリケーション (shoken-webapp)

日本株の証券情報を検索・管理する Web アプリケーションです。  
フロントエンドは React + TypeScript、バックエンドは Rust + Axum で構成しています。

## 機能概要

- 銘柄コード・会社名による証券情報検索
- CSV からの明細インポート（配当金・国内株式・投資信託）
- CSV からの保有銘柄インポートとポートフォリオ表示
- Google OAuth 認証
- J-Quants API 連携
  - 決算サマリー取得
  - 配当利回り一括取得（キャッシュ付き）

## 技術スタック

### フロントエンド

- React 19 + TypeScript
- Vite
- React Router
- TanStack Query
- Tailwind CSS
- Axios
- Recharts
- Vitest + React Testing Library
- Playwright

### バックエンド

- Rust (Edition 2021) + Tokio
- Axum
- SQLx + PostgreSQL
- OAuth2（Google 認証）
- Reqwest

### インフラ

- Vercel（Frontend）
- Fly.io（Backend）
- PostgreSQL
- GitHub Actions（CI/CD）

## セットアップ

### 前提ツール

- Node.js 20 以上
- npm
- Rust（stable）
- Docker（ローカル PostgreSQL 用）

### リポジトリ取得

```bash
git clone https://github.com/zaichu/shoken-webapp.git
cd shoken-webapp
```

### 環境変数

#### Backend（`backend/.env`）

```bash
cd backend
cp .env.example .env
```

主な項目:

- 必須
  - `DATABASE_URL`（例: `postgresql://user:password@localhost:5432/shoken_db`）
- 認証を使う場合
  - `GOOGLE_CLIENT_ID`
  - `GOOGLE_CLIENT_SECRET`
- J-Quants を使う場合
  - `JQUANTS_API_KEY`
- 任意
  - `PORT`（デフォルト: `3001`）
  - `BACKEND_URL`
  - `FRONTEND_URL`
  - `CORS_ORIGINS`

#### Frontend（`frontend/.env.development.local`）

```bash
VITE_SHOKEN_WEBAPI_API_URL=http://127.0.0.1:3001
```

## ローカル起動

### 一括起動（推奨）

```bash
./scripts/start-local.sh
```

起動後:

- Frontend: `http://127.0.0.1:8080`
- Backend: `http://127.0.0.1:3001`
- DB: `postgresql://user:password@localhost:5432/shoken_db`

停止:

```bash
./stop-local.sh
# または
./scripts/stop-local.sh
```

### 個別起動

```bash
# DB
cd backend && make db-up

# Backend（起動時にマイグレーション実行）
cd backend && make run

# Frontend
cd frontend && npm run dev -- --host 127.0.0.1 --port 8080
```

## 検証コマンド

### Frontend

```bash
cd frontend
npm run lint
npx tsc --noEmit
npm test
npm run build
```

### Backend

```bash
cd backend
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build
```

## API エンドポイント

業務 API は `/api/v1` プレフィックスを持つ。プローブ（`/health`, `/ready`）は例外としてルート直下に置く。詳細は [docs/api/v1-rest-design.md](docs/api/v1-rest-design.md) を参照。

### プローブ

| Method | Path | 説明 | 認証 |
|---|---|---|---|
| GET | `/health` | 生存確認 | 不要 |
| GET | `/ready` | 起動レディネス確認 | 不要 |

### セッション・アカウント

| Method | Path | 説明 | 認証 |
|---|---|---|---|
| GET | `/api/v1/session` | 現在ユーザーのセッション取得 | 必要 |
| DELETE | `/api/v1/session` | ログアウト | 必要 |
| POST | `/api/v1/account-deletion-confirmations` | アカウント削除確認（確認クッキー発行） | 必要 |
| DELETE | `/api/v1/account` | アカウント削除（確認クッキー必須） | 必要 |
| GET | `/api/v1/oauth/google/authorize` | Google OAuth 認証開始 | 不要 |
| GET | `/api/v1/oauth/google/callback` | Google OAuth コールバック | 不要 |

### 銘柄

| Method | Path | 説明 | 認証 |
|---|---|---|---|
| GET | `/api/v1/stocks?query=7203` | 銘柄検索（コード・社名） | 不要 |
| POST | `/api/v1/stocks` | 銘柄追加 | 必要 |

### 配当金（認証必須）

| Method | Path | 説明 |
|---|---|---|
| GET | `/api/v1/dividends` | 配当金一覧取得 |
| DELETE | `/api/v1/dividends` | 配当金全削除 |
| POST | `/api/v1/dividend-import-validations` | 配当金 CSV バリデーション（DB書込なし） |
| POST | `/api/v1/dividend-imports` | 配当金 CSV インポート |
| POST | `/api/v1/dividend-per-share-estimates` | 配当利回り一括取得 |

### 国内株式明細（認証必須）

| Method | Path | 説明 |
|---|---|---|
| GET | `/api/v1/domestic-stock-transactions` | 国内株式一覧取得 |
| DELETE | `/api/v1/domestic-stock-transactions` | 国内株式全削除 |
| POST | `/api/v1/domestic-stock-import-validations` | 国内株式 CSV バリデーション |
| POST | `/api/v1/domestic-stock-imports` | 国内株式 CSV インポート |

### 投資信託明細（認証必須）

| Method | Path | 説明 |
|---|---|---|
| GET | `/api/v1/mutual-fund-transactions` | 投資信託一覧取得 |
| DELETE | `/api/v1/mutual-fund-transactions` | 投資信託全削除 |
| POST | `/api/v1/mutual-fund-import-validations` | 投資信託 CSV バリデーション |
| POST | `/api/v1/mutual-fund-imports` | 投資信託 CSV インポート |

### 保有銘柄（認証必須）

| Method | Path | 説明 |
|---|---|---|
| GET | `/api/v1/asset-balances` | 保有銘柄一覧取得 |
| PUT | `/api/v1/asset-balances` | 保有銘柄全置換（CSV 全件更新） |
| DELETE | `/api/v1/asset-balances` | 保有銘柄全削除 |
| POST | `/api/v1/asset-balance-import-validations` | 保有銘柄 CSV バリデーション |
| POST | `/api/v1/asset-balance-imports` | 保有銘柄 CSV インポート |

### マーケットデータ

| Method | Path | 説明 | 認証 |
|---|---|---|---|
| GET | `/api/v1/financial-statements?code=7203` | 決算サマリー取得 | 不要 |

> 旧ルート（`/auth/google`、`/stock/{query}`、`/dividends/all` 等）は v1 移行完了後に削除済み。
> 移行履歴は [docs/api/v1-rest-design.md](docs/api/v1-rest-design.md) の「Legacy Route Migration History」を参照。

## ディレクトリ構成

```text
shoken-webapp/
├── frontend/
│   ├── src/
│   │   ├── components/        # Atomic Design (atoms/molecules/organisms/templates)
│   │   ├── features/          # auth / stock / receipt / assetBalance / jquants
│   │   ├── hooks/
│   │   ├── pages/
│   │   ├── lib/
│   │   ├── contexts/
│   │   ├── routes/
│   │   └── styles/
│   ├── e2e/
│   └── package.json
├── backend/
│   ├── src/
│   │   ├── handlers/
│   │   ├── services/
│   │   ├── models/
│   │   ├── extractors/
│   │   ├── routes.rs
│   │   ├── db.rs
│   │   ├── config.rs
│   │   └── main.rs
│   ├── migrations/
│   ├── docker-compose.yml
│   └── Cargo.toml
├── scripts/
│   ├── start-local.sh
│   └── stop-local.sh
├── stop-local.sh              # scripts/stop-local.sh へのラッパー
└── .github/workflows/
```

## Git ブランチ運用

- 公式ルールは [`.claude/rules/03-git.md`](.claude/rules/03-git.md) を参照
- 1タスク1ブランチ（`main` 起点）で運用する
- `main` への直接コミットは禁止

## デプロイ

- Frontend: Vercel
- Backend: Fly.io
- `main` 反映後に CI/CD で自動デプロイ

公開 URL:
- https://shoken-webapp.vercel.app

## ドキュメント

| ドキュメント | 内容 |
|---|---|
| [アーキテクチャ概要](docs/architecture.md) | システム構成・認証フロー・ミドルウェアスタック・CSV設計 |
| [運用ランブック](docs/runbook.md) | ローカル起動・デプロイ・DBマイグレーション・トラブルシューティング |
| [テストガイド](docs/testing.md) | フロントエンド・バックエンド・E2Eテストの実行方法と方針 |
| [技術者倫理ガイドライン](docs/ethics.md) | データ最小化・透明性・セキュリティ・アクセシビリティの判断基準 |

## 補足

- フロントエンド詳細: `frontend/README.md`
- バックエンド詳細: `backend/README.md`

## セキュリティ

セキュリティ上の問題を発見した場合は [SECURITY.md](SECURITY.md) を参照してください。

## ライセンス

MIT

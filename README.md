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

### 認証

| Method | Path | 説明 |
|---|---|---|
| GET | `/auth/google` | Google OAuth 認証開始 |
| GET | `/auth/google/callback` | OAuth コールバック |
| GET | `/auth/me` | 現在ユーザー取得 |
| POST | `/auth/logout` | ログアウト |
| DELETE | `/auth/delete-account` | アカウント削除 |

### 銘柄

| Method | Path | 説明 | 認証 |
|---|---|---|---|
| GET | `/stock/{query}` | 銘柄検索 | 不要 |
| POST | `/stock` | 銘柄追加 | 必要 |

### 明細データ（認証必須）

| Method | Path | 説明 |
|---|---|---|
| GET | `/dividends` | 配当金一覧取得 |
| DELETE | `/dividends/all` | 配当金全削除 |
| GET | `/domestic-stocks` | 国内株式一覧取得 |
| DELETE | `/domestic-stocks/all` | 国内株式全削除 |
| GET | `/mutualfunds` | 投資信託一覧取得 |
| DELETE | `/mutualfunds/all` | 投資信託全削除 |
| GET | `/asset-balances` | 保有銘柄一覧取得 |
| POST | `/asset-balances/bulk` | 保有銘柄一括登録（全削除→再挿入） |
| DELETE | `/asset-balances/all` | 保有銘柄全削除 |

### J-Quants / 補助 API

| Method | Path | 説明 | 認証 |
|---|---|---|---|
| GET | `/jquants/fins/statements` | 決算サマリー取得 | 不要 |
| POST | `/dividends/per-share/batch` | 配当利回り一括取得 | 不要 |
| GET | `/health` | ヘルスチェック | 不要 |

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

## 補足

- フロントエンド詳細: `frontend/README.md`
- バックエンド詳細: `backend/README.md`

## セキュリティ

セキュリティ上の問題を発見した場合は [SECURITY.md](SECURITY.md) を参照してください。

## ライセンス

MIT

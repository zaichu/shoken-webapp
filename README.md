# 証券情報ウェブアプリケーション (shoken-webapp)

日本株の証券情報を検索・管理するウェブアプリケーション。
フロントエンドは React + TypeScript、バックエンドは Rust + Axum で構築。

## 機能概要

- 銘柄コード・会社名による証券情報検索
- CSV からの取引データインポート（配当金・国内株式・投資信託）
- 保有銘柄の資産管理とポートフォリオ表示
- J-Quants API 連携による決算サマリー取得
- Google OAuth 認証

## 技術スタック

### フロントエンド

- **React** 19.2 + **TypeScript** 5.9
- **Vite** 7.3（ビルド）
- **Tailwind CSS** 4.1
- **React Router** 7.13
- **TanStack Query** 5.90
- **Axios**（HTTP クライアント）
- **Recharts**（チャート）
- **Vitest** + **React Testing Library**（テスト）
- **Playwright**（E2E テスト）

### バックエンド

- **Rust** 1.93 (Edition 2021)
- **Axum** 0.8（Web フレームワーク）
- **SQLx** 0.8 + **PostgreSQL**
- **Tokio** 1.49（非同期ランタイム）
- **OAuth2** 5.0（Google 認証）
- **Reqwest**（HTTP クライアント）

### インフラ

- **Vercel**（フロントエンド）
- **Fly.io**（バックエンド）
- **PostgreSQL**（Fly.io 上）
- **GitHub Actions**（CI/CD）

## 開発環境のセットアップ

### 必要なツール

- Rust（rustup 経由）
- Node.js 20 以上 + npm
- Docker（ローカル DB 用）

### インストール

```bash
git clone https://github.com/zaichu/shoken-webapp.git
cd shoken-webapp
```

### 環境変数

バックエンド（`backend/.env`）:

```bash
cd backend
cp .env.example .env
# .env を編集
```

```bash
# 必須
DATABASE_URL=postgres://user:password@localhost:5432/shoken_db

# Google OAuth（認証機能に必要）
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret
FRONTEND_URL=http://localhost:8080

# J-Quants API（決算サマリー取得に必要）
JQUANTS_API_KEY=your_api_key
```

フロントエンド（`frontend/.env.development.local`）:

```bash
VITE_SHOKEN_WEBAPI_API_URL=http://localhost:3001
```

## ローカル起動

### 一括起動（推奨）

```bash
./scripts/start-local.sh
```

DB → バックエンド → フロントエンドを順に起動:
- フロントエンド: http://127.0.0.1:8080
- バックエンド: http://127.0.0.1:3001

停止:

```bash
./scripts/stop-local.sh
```

### 個別起動

```bash
# DB
cd backend && make db-up

# バックエンド
cd backend && make run

# フロントエンド
cd frontend && npm run dev
```

## テスト

```bash
# フロントエンド
cd frontend
npm run lint        # ESLint
npx tsc --noEmit    # 型チェック
npm test            # Vitest

# バックエンド
cd backend
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## デプロイ

### バックエンド（Fly.io）

```bash
cd backend && make deploy
```

GitHub Actions で `main` ブランチへのマージ時に自動デプロイ。

### フロントエンド（Vercel）

Vercel と連携済み。`main` ブランチへのマージ時に自動デプロイ。
https://shoken-webapp.vercel.app

## プロジェクト構成

```
shoken-webapp/
├── frontend/                   # React フロントエンド
│   ├── src/
│   │   ├── components/         # UI コンポーネント (atoms/molecules/organisms/templates)
│   │   ├── features/           # 機能モジュール (auth, stock, receipt, jquants)
│   │   ├── hooks/              # カスタムフック
│   │   ├── pages/              # ページコンポーネント
│   │   ├── lib/                # API クライアント、ユーティリティ
│   │   ├── contexts/           # React Context
│   │   ├── styles/             # Tailwind CSS
│   │   └── types/              # 型定義
│   ├── e2e/                    # Playwright E2E テスト
│   └── package.json
├── backend/                    # Rust/Axum バックエンド
│   ├── src/
│   │   ├── handlers/           # リクエストハンドラー
│   │   ├── models/             # データモデル
│   │   ├── services/           # ビジネスロジック
│   │   ├── extractors/         # カスタムエクストラクター
│   │   ├── routes.rs           # ルーティング定義
│   │   ├── config.rs           # 設定
│   │   ├── db.rs               # DB 接続・マイグレーション
│   │   ├── middleware.rs       # CORS ミドルウェア
│   │   └── main.rs
│   ├── migrations/             # SQLx マイグレーション
│   ├── tests/                  # 統合テスト
│   └── Cargo.toml
├── scripts/                    # ローカル起動/停止スクリプト
└── .github/workflows/          # CI/CD
```

## API エンドポイント

### 認証

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/auth/google` | Google OAuth 認証開始 |
| GET | `/auth/google/callback` | OAuth コールバック |
| GET | `/auth/me` | ユーザー情報取得 |
| POST | `/auth/logout` | ログアウト |
| DELETE | `/auth/delete-account` | アカウント削除 |

### 証券情報

| メソッド | パス | 説明 | 認証 |
|---------|------|------|------|
| GET | `/stock/{query}` | 銘柄検索 | 不要 |
| POST | `/stock` | 銘柄追加 | 必要 |

### 配当金（認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/dividends` | 一覧取得 |
| POST | `/dividends/bulk` | 一括追加 |
| DELETE | `/dividends/all` | 全削除 |

### 国内株式（認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/domestic-stocks` | 一覧取得 |
| POST | `/domestic-stocks/bulk` | 一括追加 |
| DELETE | `/domestic-stocks/all` | 全削除 |

### 投資信託（認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/mutualfunds` | 一覧取得 |
| POST | `/mutualfunds/bulk` | 一括追加 |
| DELETE | `/mutualfunds/all` | 全削除 |

### 保有銘柄（認証必須）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/asset-balances` | 一覧取得 |
| POST | `/asset-balances/bulk` | 一括追加（既存は更新） |
| DELETE | `/asset-balances/all` | 全削除 |

### その他

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/jquants/fins/statements` | 決算サマリー取得 |
| GET | `/health` | ヘルスチェック |

## ライセンス

MIT

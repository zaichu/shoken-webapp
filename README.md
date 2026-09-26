# 証券情報ウェブアプリケーション (shoken-webapp)

日本株の証券情報を検索・管理する Web アプリケーションです。フロントエンドは Rust + Leptos (CSR)、バックエンドは Rust + Axum です。

## 機能

- 銘柄コード・会社名による証券情報検索
- CSV からの取引明細・保有銘柄のインポートと管理
- Google OAuth 認証
- J-Quants API 連携

## 構成

- `frontend/`: Leptos、Trunk、Tailwind CSS、Playwright。Vercel に配信
- `backend/`: Axum、SQLx、PostgreSQL。Fly.io に配信
- `docs/openapi.json`: API 契約
- `.github/workflows/`: CI とデプロイ

## ローカル開発

Rust 1.96.0、`wasm32-unknown-unknown`、Trunk 0.21.4、Node.js 22、npm、Docker が必要です。

```bash
cd frontend
npm ci
cd ../backend
cp .env.example .env
cd ..
./scripts/start-local.sh
```

`start-local.sh` は DB → backend → Leptos の順に起動します。フロントエンドは `http://127.0.0.1:8081`、バックエンドは `http://127.0.0.1:3001` です。停止には `./scripts/stop-local.sh` を使います。

## 検証

```bash
cd frontend
cargo fmt --check
cargo clippy --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo test
trunk build --release
npx playwright test --config playwright.leptos.config.ts
npx playwright test --config playwright.vercel.config.ts
```

```bash
cd backend
cargo fmt --check
env SQLX_OFFLINE=true cargo clippy -- -D warnings
env SQLX_OFFLINE=true cargo test
```

詳細は [Leptos の README](frontend/README.md) と [テストガイド](docs/testing.md) を参照してください。

## ドキュメント

- [アーキテクチャ概要](docs/architecture.md)
- [運用ランブック](docs/runbook.md)
- [API ドキュメント](docs/api/v1-rest-design.md)
- [技術者倫理ガイドライン](docs/ethics.md)

## セキュリティ

セキュリティ上の問題は [SECURITY.md](SECURITY.md) に従って報告してください。

## ライセンス

MIT

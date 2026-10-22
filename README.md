# 証券情報ウェブアプリケーション (shoken-webapp)

日本株の証券情報を検索・管理する Web アプリケーションです。フロントエンドは Rust + Leptos (CSR)、バックエンドは Rust + Axum です。

## 機能

- 銘柄コード・会社名による証券情報検索
- CSV からの取引明細・保有銘柄のインポートと管理
- Google OAuth 認証
- J-Quants API 連携

## 構成

- `frontend/`: Leptos、Trunk、Tailwind CSS、Playwright。Cloudflare Pages に配信
- `backend/`: Axum、tokio-postgres、PostgreSQL。Cloudflare Workers に配信
- `docs/openapi.json`: API 契約
- `.github/workflows/`: CI とデプロイ

## ローカル開発

Rust（版はルートの `rust-toolchain.toml` を参照）、`wasm32-unknown-unknown`、Trunk 0.21.4、Node.js 22、npm、Docker が必要です。

```bash
cd frontend
npm ci
cd ../backend
cp .dev.vars.example .dev.vars
cd ..
./scripts/start-local.sh
```

`start-local.sh` は DB → backend（wrangler dev）→ Leptos の順に起動します。フロントエンドは `http://127.0.0.1:8081`、バックエンドは `http://127.0.0.1:8787` です。停止には `./scripts/stop-local.sh` を使います。

DB ドキュメント（テーブル一覧と ER 図）の生成には開発用ツールの `tbls` が必要（`brew install tbls` または `go install github.com/k1LoW/tbls@v1.96.1`、1.96 系）。生成は `./scripts/gen-db-docs.sh`（Docker と Rust が必要）で行い、migration を変えたら `docs/db/` を更新してコミットします。

## 検証

```bash
cd frontend
cargo fmt --check
cargo clippy --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo test
trunk build --release
npx playwright test --config playwright.leptos.config.ts
npx playwright test --config playwright.deploy.config.ts
```

```bash
cd backend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
worker-build --release
```

詳細は [Leptos の README](frontend/README.md) と [テストガイド](docs/testing.md) を参照してください。

## ドキュメント

- [アーキテクチャ概要](docs/architecture.md)
- [DB スキーマ（テーブル一覧と ER 図）](docs/db/README.md)
- [運用ランブック](docs/runbook.md)
- [API ドキュメント](docs/api/v1-rest-design.md)
- [技術者倫理ガイドライン](docs/ethics.md)

## セキュリティ

セキュリティ上の問題は [SECURITY.md](SECURITY.md) に従って報告してください。

## ライセンス

MIT

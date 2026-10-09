# 運用ランブック

## ローカル開発環境の起動

```bash
# 1. リポジトリをクローン
git clone https://github.com/zaichu/shoken-webapp.git
cd shoken-webapp

# 2. 環境変数を設定
(cd backend && cp .env.example .env)  # DATABASE_URL 等を設定
(cd frontend && npm ci)

# 3. 一括起動（推奨）
./scripts/start-local.sh
```

起動後のアクセス先:
- フロントエンド: http://127.0.0.1:8081
- バックエンド: http://127.0.0.1:3001
- DB: `postgresql://user:password@localhost:5432/shoken_db`

停止: `./scripts/stop-local.sh`

## デプロイ

`main` ブランチへのマージで CI/CD が自動実行されます。

| サービス | デプロイ先 | トリガー |
|---|---|---|
| フロントエンド | Cloudflare Pages | Frontend CI（`frontend.yml`）成功後に `deploy-cloudflare-pages.yml` が呼び出されて自動デプロイ（frontend/shared 変更時） |
| フロントエンド（プレビュー） | Cloudflare Pages / Vercel preview | PR 作成・更新時に `deploy-cloudflare-pages.yml` / `deploy-frontend.yml` が独立ビルド・配信（投稿者が OWNER/MEMBER/COLLABORATOR の場合のみ）。Frontend CI の成功は待たない。URL は PR コメントに投稿される |
| フロントエンド（旧本番） | Vercel | `LEPTOS_PRODUCTION_ENABLED=false` の間、`deploy-frontend.yml` は preview としてのみデプロイする |
| バックエンド | Cloudflare Workers | main push（`deploy-cloudflare-worker.yml`） |
| バックエンド（旧・切り戻し用に稼働中） | Fly.io | main push（`deploy-backend.yml`） |

`fly.toml` はリポジトリルートに置く(Docker build context が `shared/` を含むルートのため)。

### 手動デプロイ（緊急時）

```bash
# バックエンド（Cloudflare Workers。secrets/vars の注入を含むため CI 経路を使う）
gh workflow run deploy-cloudflare-worker.yml

# バックエンド（旧: Fly.io）
(cd backend && make deploy)

# フロントエンド（Cloudflare Pages）
gh workflow run deploy-cloudflare-pages.yml

# フロントエンド（旧: Vercel CLI。リポジトリルートで実行）
vercel pull --yes --environment=production
vercel build --prod
vercel deploy --prebuilt --prod
```

## ヘルスチェック

```bash
# バックエンド（本番: Cloudflare Workers）
curl https://shoken-backend.zaitomo41.workers.dev/health

# バックエンド（旧: Fly.io。切り戻し用に稼働中）
curl https://shoken-backend.fly.dev/health

# ログ確認（Fly.io）
fly logs --app shoken-backend

# ログ確認（Cloudflare Workers。wrangler の認証が必要）
(cd backend && wrangler tail)
```

## データベースマイグレーション

```bash
# マイグレーション作成
(cd backend && sqlx migrate add <migration_name>)

# マイグレーション実行（起動時に自動実行）
(cd backend && make run)

# オフラインモード用 sqlx-data 更新
(cd backend && make sqlx-prepare)
```

## 環境変数（本番）

Fly.io Secrets で管理（旧構成・切り戻し用に残す）:

```bash
fly secrets set DATABASE_URL="postgresql://..."
fly secrets set GOOGLE_CLIENT_ID="..."
fly secrets set GOOGLE_CLIENT_SECRET="..."
fly secrets set FRONTEND_URL="https://shoken-webapp.pages.dev"
fly secrets set BACKEND_URL="https://shoken-backend.fly.dev"
```

Cloudflare Workers 側の vars/secrets は `deploy-cloudflare-worker.yml` が GitHub
secrets/vars から注入する（`wrangler secret put` と `wrangler deploy --var`）。
`FRONTEND_URL` はリポジトリ variable で上書きでき、未設定時の既定は
`https://shoken-webapp.pages.dev`。CORS の許可 origin は `CORS_ORIGINS` var 未設定時に
コード既定値（`backend/src/config.rs`）が使われ、本番では `shoken-webapp.pages.dev` と
`shoken-webapp.vercel.app` を許可する。

## Google Cloud OAuth 設定

Google Cloud Console で以下の **Authorized redirect URIs** を登録する:

| 環境 | URI |
|---|---|
| 本番（Cloudflare Workers） | `https://shoken-backend.zaitomo41.workers.dev/api/v1/oauth/google/callback` |
| 旧本番（Fly.io。切り戻し用に登録したまま残す） | `https://shoken-backend.fly.dev/api/v1/oauth/google/callback` |
| ローカル | `http://localhost:3001/api/v1/oauth/google/callback` |

> **注意**: 旧 `https://shoken-backend.fly.dev/auth/google/callback` は現行 API では使用しない。
> Google Cloud Console に登録している場合は削除する。

## バックエンド接続先の切り替え（Workers ⇄ Fly.io）

frontend が参照する本番 API の正本は `frontend/vercel.json` の CSP `connect-src` です。
`frontend/scripts/prepare-vercel-dist.mjs` がそこから `index.html` の `shoken-api-origin`
meta と Pages 配信用の `_headers` を生成するため、接続先の変更は `connect-src` の
1 箇所だけを直して main にマージすれば Pages 本番へ反映されます。

現在の接続先: `https://shoken-backend.zaitomo41.workers.dev`（#1210 段階7）

### Fly.io への切り戻し

Fly.io の machine は撤去していないため、frontend の接続先を戻すだけで復帰できます。

1. `frontend/vercel.json` の CSP `connect-src` を `https://shoken-backend.fly.dev` に戻す
   （この切り替え PR の revert でも同じ）
2. main にマージする。`deploy-cloudflare-pages.yml` が Pages 本番へ反映する
3. 反映を確認する: `curl -s https://shoken-webapp.pages.dev/` の `shoken-api-origin` meta と
   `Content-Security-Policy` の `connect-src` が `https://shoken-backend.fly.dev` を指すこと
4. Fly 側の `FRONTEND_URL` / `CORS_ORIGINS` が `https://shoken-webapp.pages.dev` を
   許可・指向しているか不明な場合は、`fly secrets set` で再投入して `fly deploy` する
   （secret の値は読み出せないため、怪しければ再設定する）。Google OAuth の承認済み
   リダイレクト URI は Fly.io 側も登録したままにしてあるため追加作業は不要

Worker 側の `FRONTEND_URL` / `CORS_ORIGINS` は `pages.dev` を指したままでよい
（どちらの backend を指すかを持っているのは frontend 側だけ）。

## セキュリティインシデント対応

1. [SECURITY.md](../SECURITY.md) の手順に従って報告を受理
2. 重大度に応じて 30〜90 日以内に修正

脆弱性が発見された場合:
```bash
# 依存関係の脆弱性チェック
(cd backend && cargo audit)
(cd frontend && npm audit --audit-level=high)
```

## PR マージ後のクリーンアップ

worktree を使った作業ブランチをマージした後は、以下の順序で後片付けをする。
ローカルブランチ削除を先に実行すると worktree がそのブランチを参照中のためエラーになる場合があるので、
必ず **worktree 削除を先に行う**こと。
worktree の配置先などの運用ルールは `.claude/rules/03-git.md` を正本とする。

```bash
# 1. PR をマージ（GitHub 側でブランチを削除してもよい）
gh pr merge <PR番号> --squash

# 2. main を最新化
git switch main
git pull --ff-only origin main

# 3. worktree を削除（使用中ブランチを手放す）
git worktree remove ../.worktrees/<repo>-<topic>

# 4. ローカルブランチを削除（squash merge 済み短期ブランチのみ）
git branch -d <branch-name>

# 5. リモートブランチを削除（gh pr merge --delete-branch を使わなかった場合）
git push origin --delete <branch-name>

# 6. リモート参照を整理
git fetch origin --prune
```

> **注意**: `gh pr merge --delete-branch` はリモートブランチを削除するが、
> ローカルに worktree が残っている状態ではブランチ削除が失敗する。
> 上記の順序（merge → main pull → worktree remove → branch -d → push delete → fetch prune）を守ること。
> `git branch -d` が拒否された場合は、PR が squash merge 済みで main に取り込まれていることを確認してから個別判断する。

## Dependabot PR 対応

毎週月曜日に Dependabot PR が作成されます。Dependabot の PR は Issue の紐づけを免除されますが、未解決コメントは通常どおり PR gate の対象です。

- **パッチ・マイナー更新**: `dependabot-auto-merge.yml` が `gh pr merge --auto --squash` を設定し、CI が green になれば自動マージされます
- **自動マージ後のデプロイ**: `GITHUB_TOKEN` によるマージでは push イベントが発火しないため、main への push CI / デプロイは走りません。frontend/shared 変更を含むマージでは `dependabot-auto-merge.yml` がマージ完了後に Frontend CI を自動 dispatch します（デプロイは CI 成功後に CI 側から呼び出されます）。backend の依存がマージされたら `gh workflow run deploy-backend.yml` で手動デプロイしてください
- **メジャー更新**: 自動マージしません。エージェントが破壊的変更を確認して対応します
- **セキュリティ更新**: `priority: P1` として扱い、優先して対応します
- **CI が red の PR**: 失敗原因を調査して対応方針を決定（修正 / 保留 / close）
- **生成物以外のコードまで直す大きな修正になった場合**: 作業前に Issue を作って PR に紐づけます

## よくあるトラブル

### DB 接続失敗

```bash
(cd backend && make db-up)  # Docker で PostgreSQL を起動
```

### バックエンドビルドエラー

```bash
(cd backend && cargo check)  # コンパイルエラー確認
(cd backend && cargo fmt)    # フォーマット修正
```

### フロントエンドのビルドエラー

```bash
(cd frontend && cargo clippy --all-targets --target wasm32-unknown-unknown -- -D warnings)
(cd frontend && trunk build --release)
```

### OpenAPI スキーマの不一致

```bash
bash scripts/check-openapi.sh  # 再生成して差分確認
```

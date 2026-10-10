# 運用ランブック

## ローカル開発環境の起動

```bash
# 1. リポジトリをクローン
git clone https://github.com/zaichu/shoken-webapp.git
cd shoken-webapp

# 2. シークレットを設定（OAuth/J-Quants を使う場合）
(cd backend && cp .dev.vars.example .dev.vars)
(cd frontend && npm ci)

# 3. 一括起動（推奨）
./scripts/start-local.sh
```

起動後のアクセス先:
- フロントエンド: http://127.0.0.1:8081
- バックエンド: http://127.0.0.1:8787（wrangler dev）
- DB: `postgresql://user:password@localhost:5432/shoken_db`

停止: `./scripts/stop-local.sh`

## デプロイ

`main` ブランチへのマージで CI/CD が自動実行されます。

| サービス | デプロイ先 | トリガー |
|---|---|---|
| フロントエンド | Cloudflare Pages（`https://shoken-webapp.pages.dev`） | Frontend CI（`frontend.yml`）成功後に `deploy-cloudflare-pages.yml` が呼び出されて自動デプロイ（frontend/shared 変更時）。手動反映は main での `deploy-cloudflare-pages.yml` workflow_dispatch |
| フロントエンド（プレビュー） | Cloudflare Pages preview | PR 作成・更新時に `deploy-cloudflare-pages.yml` が独立ビルド・配信（投稿者が OWNER/MEMBER/COLLABORATOR の場合のみ）。Frontend CI の成功は待たない。URL は PR コメントに投稿される |
| バックエンド | Cloudflare Workers（`https://shoken-backend.zaitomo41.workers.dev`） | main push（`deploy-cloudflare-worker.yml`） |

### 手動デプロイ（緊急時）

```bash
# バックエンド（Cloudflare Workers。secrets/vars の注入を含むため CI 経路を使う）
gh workflow run deploy-cloudflare-worker.yml

# フロントエンド（Cloudflare Pages。main で workflow_dispatch）
gh workflow run deploy-cloudflare-pages.yml --ref main
```

## ヘルスチェック

```bash
# バックエンド（Cloudflare Workers）
curl https://shoken-backend.zaitomo41.workers.dev/health

# ログ確認（Cloudflare Workers。wrangler の認証が必要）
(cd backend && wrangler tail)
```

## データベースマイグレーション

```bash
# マイグレーション実行（ローカル Docker DB へ）
(cd backend && make migrate-local)
```

新規マイグレーションは `backend/migrations/NNNN_<name>.sql`（次番号）を作り、
`backend/src/db/migrate.rs` の `MIGRATION_FILES` に `include_str!` で登録する。
詳細は `backend/migrations/README.md`。

## 環境変数（本番）

Cloudflare Workers 側の vars/secrets は `deploy-cloudflare-worker.yml` が GitHub
secrets/vars から注入する（`wrangler secret put` と `wrangler deploy --var`）。
`FRONTEND_URL` はリポジトリ variable で上書きでき、未設定時の既定は
`https://shoken-webapp.pages.dev`。CORS の許可 origin は `CORS_ORIGINS` var 未設定時に
コード既定値（`backend/src/config.rs`）が使われ、本番では `shoken-webapp.pages.dev` を
許可する。

## Google Cloud OAuth 設定

Google Cloud Console で以下の **Authorized redirect URIs** を登録する:

| 環境 | URI |
|---|---|
| 本番（Cloudflare Workers） | `https://shoken-backend.zaitomo41.workers.dev/api/v1/oauth/google/callback` |
| ローカル | `http://localhost:8787/api/v1/oauth/google/callback` |

> **注意**: 旧環境の `https://shoken-backend.fly.dev/...` は現行 API では使用しない。
> Google Cloud Console に登録している場合は削除する。

## バックエンド接続先の切り替え

frontend が参照する本番 API の正本は `frontend/_headers` の CSP `connect-src` です。
`frontend/scripts/prepare-dist.mjs` がそこから `index.html` の `shoken-api-origin`
meta へ反映するため、接続先の変更は `connect-src` の
1 箇所だけを直して main にマージすれば Pages 本番へ反映されます。

現在の接続先: `https://shoken-backend.zaitomo41.workers.dev`（#1210 段階7）

反映確認: `curl -s https://shoken-webapp.pages.dev/` の `shoken-api-origin` meta と
`Content-Security-Policy` の `connect-src` が `https://shoken-backend.zaitomo41.workers.dev`
を指すこと。

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
- **自動マージ後のデプロイ**: `GITHUB_TOKEN` によるマージでは push イベントが発火しないため、main への push CI / デプロイは走りません。frontend/shared 変更を含むマージでは `dependabot-auto-merge.yml` がマージ完了後に Frontend CI を自動 dispatch します（デプロイは CI 成功後に CI 側から呼び出されます）。backend の依存がマージされたら `gh workflow run deploy-cloudflare-worker.yml` で手動デプロイしてください
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

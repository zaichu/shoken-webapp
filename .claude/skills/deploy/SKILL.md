---
name: deploy
description: |
  Fly.io (backend) と Cloudflare Pages (frontend) へのデプロイ。
  Use when: デプロイ、本番反映、fly deploy を依頼された時。
---

# デプロイ

## 重要: 本番デプロイの経路

- backend: `main` へのマージ(`backend/**`・`shared/**` 変更)で `deploy-backend.yml` が自動デプロイ
- frontend: 本番は Cloudflare Pages (`https://shoken-webapp.pages.dev`)。`main` への frontend/shared 変更 push で `frontend.yml` 成功後に `deploy-cloudflare-pages.yml` が自動デプロイ。手動反映は `main` に対する同ワークフローの workflow_dispatch
- 旧環境 Vercel (`https://shoken-webapp.vercel.app`) は切り戻し用に残置(削除しない)。`deploy-frontend.yml` は `LEPTOS_PRODUCTION_ENABLED=true` のときのみ本番へ出る

ブランチ運用の基準は `.claude/rules/03-git.md` を参照する。

## バックエンド (Fly.io)

### 自動デプロイ（推奨）
- `main` ブランチへのマージで自動デプロイ（GitHub Actions）
- `backend/**`・`shared/**`・`fly.toml` の変更時にトリガー
- **手動デプロイは不要**

### 手動デプロイ（緊急時のみ）

`fly.toml` はリポジトリルートに置く（backend の Docker build context が `shared/` を含むルートのため）。リポジトリルートで実行する:

```bash
flyctl deploy --remote-only
```

### ログ確認
```bash
flyctl logs -a shoken-backend
flyctl status -a shoken-backend
```

## フロントエンド (Cloudflare Pages)

### 本番デプロイ

- 自動: `main` への frontend/shared 変更 push → `frontend.yml` 成功後に `deploy-cloudflare-pages.yml` が検証済みの `cloudflare-dist` 成果物をそのままデプロイ(二重ビルドしない)
- 手動: `deploy-cloudflare-pages.yml` を `main` ブランチで workflow_dispatch 実行(独立ビルド経路)
- PR では同ワークフローが `pull_request` で起動し、preview を `<branch>.shoken-webapp.pages.dev` に配信して URL を PR コメントに投稿(投稿者が OWNER/MEMBER/COLLABORATOR の場合のみ)

配信設定の正本は `frontend/vercel.json`。`prepare-vercel-dist.mjs` が `_headers`・`_redirects` を生成し、Pages は dist 直下のそれらを読む。Vercel の Git 連携ビルドは `frontend/vercel.json` の ignoreCommand で常にスキップされる。

### フロントエンド旧環境 (Vercel、切り戻し用に残置)

本番反映は `LEPTOS_PRODUCTION_ENABLED=true` のときだけ。再開する場合:

- `deploy-frontend.yml` を `main` ブランチで workflow_dispatch 実行(production)
- またはリポジトリのルートで Vercel CLI を実行する:

```bash
vercel pull --yes --environment=production
vercel build --prod
vercel deploy --prebuilt --prod
```

## 開発フロー

1. `main` から作業ブランチを作成して実装
2. ローカルでテスト（cargo test, playwright）
3. 作業ブランチ -> `main` の PR を作成してマージ
4. backend は自動デプロイ実行（手動操作不要）
5. frontend も main へのマージで自動デプロイ(frontend/shared 変更時)。手動反映は `deploy-cloudflare-pages.yml` の workflow_dispatch
6. 本番で動作確認

## デプロイ前チェックリスト

- [ ] ローカルテストがすべてパス
- [ ] ビルドが成功
- [ ] 環境変数が正しく設定されている
- [ ] マイグレーションが必要な場合は先に実行

## ロールバック

### Fly.io
```bash
flyctl releases list -a shoken-backend
flyctl deploy -a shoken-backend --image <previous-image>
```

### Cloudflare Pages → Vercel への切り戻し
手順は `docs/runbook.md` の「切り戻し（フロントを Vercel 本番へ戻す）」を正本とする。

### Vercel (旧環境内での戻し)
Vercel ダッシュボードから以前のデプロイを選択して再デプロイ

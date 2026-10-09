---
name: deploy
description: |
  Cloudflare Workers (backend) と Cloudflare Pages (frontend) へのデプロイ。
  Use when: デプロイ、本番反映を依頼された時。
---

# デプロイ

## 重要: 本番デプロイの経路

- backend: `main` へのマージ(`backend/**`・`shared/**` 変更)で `deploy-cloudflare-worker.yml` が自動デプロイ (`https://shoken-backend.zaitomo41.workers.dev`)
- frontend: `main` への frontend/shared 変更 push で `frontend.yml` 成功後に `deploy-cloudflare-pages.yml` が自動デプロイ (`https://shoken-webapp.pages.dev`)。手動反映は `main` に対する同ワークフローの workflow_dispatch

ブランチ運用の基準は `.claude/rules/03-git.md` を参照する。

## バックエンド (Cloudflare Workers)

### 自動デプロイ（推奨）
- `main` ブランチへのマージで自動デプロイ（GitHub Actions）
- `backend/**`・`shared/**`・`wrangler.toml` 等の変更時にトリガー
- **手動デプロイは不要**

### 手動デプロイ（緊急時のみ）

secrets/vars の注入を含むため CI 経路を使う:

```bash
gh workflow run deploy-cloudflare-worker.yml
```

### ログ確認（wrangler の認証が必要）
```bash
cd backend && wrangler tail
```

## フロントエンド (Cloudflare Pages)

### 本番デプロイ

- 自動: `main` への frontend/shared 変更 push → `frontend.yml` 成功後に `deploy-cloudflare-pages.yml` が検証済みの `cloudflare-dist` 成果物をそのままデプロイ(二重ビルドしない)
- 手動: `deploy-cloudflare-pages.yml` を `main` ブランチで workflow_dispatch 実行(独立ビルド経路)
- PR では同ワークフローが `pull_request` で起動し、preview を `<branch>.shoken-webapp.pages.dev` に配信して URL を PR コメントに投稿(投稿者が OWNER/MEMBER/COLLABORATOR の場合のみ)

配信設定の正本は `frontend/_headers` と `frontend/_redirects`。`prepare-dist.mjs` が `_headers` の CSP `connect-src` から `index.html` の `shoken-api-origin` meta などを生成し、Pages は dist 直下のそれらを読む。

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

### Cloudflare Workers
```bash
cd backend && wrangler rollback
```

### Cloudflare Pages
Pages ダッシュボードから以前のデプロイを選択して再デプロイ

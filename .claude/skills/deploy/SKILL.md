---
name: deploy
description: |
  Fly.io (backend) と Vercel (frontend) へのデプロイ。
  Use when: デプロイ、本番反映、fly deploy を依頼された時。
---

# デプロイ

## 重要: 本番デプロイの経路

- backend: `main` へのマージ(`backend/**` 変更)で `deploy-backend.yml` が自動デプロイ
- frontend: PR や `main` への push では自動デプロイしない(Vercel の Git 連携ビルドは `vercel-ignore-build.sh` で常にスキップ)。本番反映は `main` に対する `deploy-frontend.yml` の workflow_dispatch か、Vercel CLI の手動実行のみ

ブランチ運用の基準は `.claude/rules/03-git.md` を参照する。

## バックエンド (Fly.io)

### 自動デプロイ（推奨）
- `main` ブランチへのマージで自動デプロイ（GitHub Actions）
- `backend/**` の変更時のみトリガー
- **手動デプロイは不要**

### 手動デプロイ（緊急時のみ）
```bash
cd backend && flyctl deploy --remote-only
```

### ログ確認
```bash
flyctl logs -a shoken-backend
flyctl status -a shoken-backend
```

## フロントエンド (Vercel)

### 本番デプロイ(手動のみ)

PR や `main` への push では自動デプロイされない。本番反映は次のいずれか:

- `deploy-frontend.yml` を `main` ブランチで workflow_dispatch 実行(`LEPTOS_PRODUCTION_ENABLED=true` のとき production、それ以外は preview)
- リポジトリのルートで Vercel CLI を実行する(Vercel の Root Directory はルートからの相対で解決される):

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
5. frontend の本番反映が必要な場合は `deploy-frontend.yml` の workflow_dispatch か Vercel CLI で手動デプロイ
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
flyctl deploy --image <previous-image>
```

### Vercel
Vercel ダッシュボードから以前のデプロイを選択して再デプロイ

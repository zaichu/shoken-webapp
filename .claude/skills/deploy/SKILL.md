---
name: deploy
description: |
  Fly.io (backend) と Vercel (frontend) へのデプロイ。
  Use when: デプロイ、本番反映、fly deploy を依頼された時。
---

# デプロイ

## バックエンド (Fly.io)

### 自動デプロイ
- `main` ブランチへの push で自動デプロイ（GitHub Actions）
- `backend/**` の変更時のみトリガー

### 手動デプロイ
```bash
cd backend && flyctl deploy --remote-only
```

### 確認
```bash
flyctl status
flyctl logs
```

## フロントエンド (Vercel)

### 自動デプロイ
- `main` ブランチへのマージで自動デプロイ
- PR作成時にプレビューデプロイ

### 手動デプロイ（必要な場合）
```bash
cd frontend && vercel --prod
```

## デプロイ前チェックリスト

- [ ] テストがすべてパス
- [ ] ビルドが成功
- [ ] 環境変数が正しく設定されている
- [ ] マイグレーションが必要な場合は先に実行

## ロールバック

### Fly.io
```bash
flyctl releases list
flyctl deploy --image <previous-image>
```

### Vercel
Vercel ダッシュボードから以前のデプロイを選択して再デプロイ

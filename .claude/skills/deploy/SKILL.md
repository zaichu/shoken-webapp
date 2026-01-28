---
name: deploy
description: |
  Fly.io (backend) と Vercel (frontend) へのデプロイ。
  Use when: デプロイ、本番反映、fly deploy を依頼された時。
---

# デプロイ

## 重要: 自動デプロイが基本

main ブランチへのマージで自動デプロイが実行されるため、
**通常は手動デプロイ不要**。

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

### 自動デプロイ（推奨）
- `main` ブランチへのマージで自動デプロイ
- PR作成時にプレビューデプロイ
- **手動デプロイは不要**

### 手動デプロイ（緊急時のみ）
```bash
cd frontend && vercel --prod
```

## 開発フロー

1. ローカルでテスト（cargo test, npm test）
2. PR作成
3. レビュー・マージ
4. **自動デプロイ実行**（手動操作不要）
5. 本番で動作確認

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

# 全体開発ルール

## プロジェクト概要

- **shoken-webapp**: 日本株の証券情報を管理するウェブアプリケーション
- フロントエンド: React 19 + TypeScript + Vite
- バックエンド: Rust + Axum + SQLx + Shuttle
- データベース: PostgreSQL

## 言語・コミュニケーション

- コードのコメントは日本語で記述
- コミットメッセージは日本語で記述
- 変数名・関数名は英語で記述

## ディレクトリ構成

```
shoken-webapp/
├── frontend/          # React フロントエンド
├── backend/           # Rust バックエンド
└── .github/workflows/ # CI/CD
```

## 開発コマンド

### フロントエンド (`frontend/`)

```bash
make dev      # 開発サーバー起動 (port 8080)
make build    # ビルド
make test     # テスト実行
npm run lint  # ESLint実行
```

### バックエンド (`backend/`)

```bash
make run      # 開発サーバー起動 (port 3001)
make test     # テスト実行
make deploy   # Shuttleへデプロイ
```

## 環境変数

### フロントエンド

- `.env.local` - ローカル開発用
- `.env.development.local` - 開発環境用
- `VITE_` プレフィックスが必要

### バックエンド

- `Secrets.dev.toml` - ローカル開発用
- `Secrets.toml` - 本番用 (gitignore)
- Shuttle SecretStore で管理

## Git運用

- メインブランチ: `main`
- 開発ブランチ: `develop`
- PRは `main` ブランチへ作成

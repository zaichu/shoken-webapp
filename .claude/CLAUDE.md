# CLAUDE.md

日本語で必ず回答してください。
このファイルは Claude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## プロジェクト概要

**shoken-webapp** は日本株の証券情報を管理するウェブアプリケーションです。

- **フロントエンド**: React 19 + TypeScript + Vite
- **バックエンド**: Rust + Axum + SQLx + Fly.io
- **データベース**: PostgreSQL (Neon)
- **認証**: Google OAuth

## プロジェクト構造

```
shoken-webapp/
├── frontend/                    # React フロントエンド
│   ├── CLAUDE.md                # フロントエンド詳細ガイド
│   └── .claude/rules/           # フロントエンド固有ルール
│       ├── 00-frontend.md
│       ├── 01-testing.md
│       └── 02-security.md
├── backend/                     # Rust バックエンド
│   ├── CLAUDE.md                # バックエンド詳細ガイド
│   └── .claude/rules/           # バックエンド固有ルール
│       ├── 00-backend.md
│       ├── 01-testing.md
│       └── 02-security.md
├── .claude/
│   ├── CLAUDE.md                # プロジェクト概要（このファイル）
│   └── rules/                   # 共通ルール
│       ├── 00-general.md        # 全体ルール
│       ├── 01-testing.md        # テスト共通ルール
│       ├── 02-security.md       # セキュリティ共通ルール
│       └── 03-git.md            # Git/PRルール
└── .github/workflows/           # CI/CD
```

## クイックスタート

### フロントエンド開発

```bash
cd frontend
make dev      # 開発サーバー起動 (port 8080)
make build    # ビルド
npm test      # テスト実行
npm run lint  # ESLint実行
```

### バックエンド開発

```bash
cd backend
make run      # 開発サーバー起動
make test     # テスト実行
make deploy   # Fly.ioへデプロイ
```

## 主要機能

- **CSVインポート**: エンコーディング自動検出付き取引データインポート
- **株式検索**: 銘柄コード/名前による日本株検索
- **取引履歴管理**: 国内株式、投資信託、配当の分析
- **税計算**: 日本の税率内蔵（20.315%）
- **J-Quants API連携**: 金融データ取得

## 開発ガイドライン

### コーディング規約

- コードコメント: 日本語
- コミットメッセージ: 日本語
- 変数名・関数名: 英語

### アーキテクチャ

- **フロントエンド**: Atomic Design パターン
- **バックエンド**: ドメイン駆動設計

詳細は `.claude/rules/` および各ディレクトリの `CLAUDE.md` を参照してください。

## Git運用

- メインブランチ: `main`
- 開発ブランチ: `develop`
- PRは `main` ブランチへ作成

## 環境変数

### フロントエンド

- `.env.local` - ローカル開発用
- `VITE_` プレフィックスが必要

### バックエンド

- `.env` - ローカル開発用 (gitignore)
- Fly.io Secrets で本番環境を管理

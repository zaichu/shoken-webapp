# 証券情報ウェブアプリケーション (shoken-webapp)

このプロジェクトは、株式情報を検索・表示するためのウェブアプリケーションです。フロントエンドとバックエンドの両方にRustを使用し、フロントエンドはWebAssembly（Yewフレームワーク）、バックエンドはAxumを採用しています。

## 機能概要

- 証券情報の検索と表示
- CSVからの取引データのインポート
- 実現損益の計算と表示
- 各種証券情報サイトへのリンク生成
- Google OAuth認証（開発中）
- データの永続化と取引履歴管理（開発中）

## デモ

以下のリンクからデモアプリをご確認いただけます。

[デモサイト](https://zaichu.github.io/shoken-webapp/)

## アーキテクチャ

このプロジェクトは以下のコンポーネントで構成されています：

- **フロントエンド**: Rust + Yew + WebAssembly (旧shoken-webapp)
- **バックエンド**: Rust + Axum + SQLx + PostgreSQL (旧shoken-webapp-api)
- **デプロイ**: Shuttle (バックエンド)

## 技術スタック

### フロントエンド
- **Rust** (1.70以上)
- **Yew** (0.21): Rustベースのフロントエンドフレームワーク
- **WebAssembly**: パフォーマンス向上のための技術
- **Yew Router**: SPA内の画面遷移
- **Gloo**: WebブラウザAPIへのRustバインディング

### バックエンド
- **Rust** (1.70以上)
- **Axum**: 高性能なWebフレームワーク
- **SQLx**: 非同期SQLツールキット
- **PostgreSQL**: データベース
- **Shuttle**: Rustアプリのデプロイプラットフォーム

## 開発環境のセットアップ

### 必要なツール

- Rust (1.70以上)
- wasm-pack
- cargo
- PostgreSQL (ローカル開発用)
- Node.js と npm (オプション: フロントエンド開発用)

### インストール手順

1. Rustとツールチェーンのインストール:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cargo install cargo-shuttle  # Shuttleデプロイ用
```

2. リポジトリのクローン:

```bash
git clone https://github.com/yourusername/shoken-webapp.git
cd shoken-webapp
```

### 環境変数の設定

バックエンドの`.env`ファイルを作成:

```bash
cd backend
touch .env
```

`.env`ファイルに以下の変数を設定:

```
DATABASE_URL=postgres://username:password@localhost:5432/shoken_db
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret
FRONTEND_URL=http://localhost:8080
```

## ビルドと実行

### Makefileによる簡易ビルド

プロジェクトのルートディレクトリから以下のコマンドを実行できます:

```bash
# 全体をビルド
make build

# フロントエンドのみビルド
make build-frontend

# バックエンドのみビルド
make build-backend

# バックエンドサーバーを実行
make run-backend

# ビルド成果物をクリーン
make clean
```

### 手動ビルド手順

#### フロントエンドのビルド

```bash
cd frontend
wasm-pack build --target web
```

#### バックエンドの実行

```bash
cd backend
cargo run
```

アプリケーションは http://localhost:8000 で起動します。

## テスト実行

プロジェクト全体のテストを実行:

```bash
make test
```

個別にテストを実行:

```bash
# フロントエンドのテスト
cd frontend
wasm-pack test --headless --chrome

# バックエンドのテスト
cd backend
cargo test
```

## デプロイ

Shuttleを使用してバックエンドをデプロイ:

```bash
make deploy
```

デプロイの状態を確認:

```bash
make status
```

## プロジェクト構造

```
shoken-webapp/
├── frontend/        # Yew + WebAssemblyフロントエンド
│   ├── src/         # フロントエンドソースコード
│   ├── static/      # 静的アセット
│   └── Cargo.toml   # フロントエンド依存関係
├── backend/         # Axumバックエンド
│   ├── src/         # バックエンドソースコード
│   ├── migrations/  # SQLxデータベースマイグレーション
│   └── Cargo.toml   # バックエンド依存関係
├── Cargo.toml       # ワークスペース設定
└── Makefile         # ビルド/デプロイコマンド
```

## 貢献方法

プロジェクトへの貢献を歓迎します。バグの報告や機能追加の提案は、Issueを作成してください。

## ライセンス

MIT

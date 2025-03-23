# 証券情報ウェブアプリ (shoken-webapp)

このプロジェクトは、既存のshoken-webapp-wasmとshoken-webapp-apiを統合した証券情報表示用のウェブアプリケーションです。フロントエンドとバックエンドの両方にRustを使用しています。

## プロジェクト構成

- `frontend/`: WebAssembly部分（旧shoken-webapp-wasm）
- `backend/`: API部分（旧shoken-webapp-api）
- `common/`: 共通コード

## 機能

- 証券情報の表示
- 証券情報の検索
- Google OAuth認証（開発中）
- 取引履歴管理（開発中）

## 開発環境のセットアップ

### 必要なツール

- Rust（1.60以上）
- wasm-pack
- cargo
- Node.js と npm（開発用）

### インストール手順

1. Rustと関連ツールをインストール:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

2. リポジトリをクローン:

```bash
git clone https://github.com/yourusername/shoken-webapp.git
cd shoken-webapp
```

### ビルドと実行

#### フロントエンドのビルド

```bash
cd frontend
wasm-pack build --target web
# ビルド成果物を静的ディレクトリにコピー
cp -r pkg/* static/
```

#### バックエンドの環境変数設定

```bash
cd ../backend
# .envファイルを作成
echo "GOOGLE_CLIENT_ID=your_client_id" > .env
echo "GOOGLE_CLIENT_SECRET=your_client_secret" >> .env
echo "FRONTEND_URL=http://localhost:8080" >> .env
```

#### バックエンドの実行

```bash
cd ../backend
cargo run
```

その後、ブラウザで http://localhost:8080 にアクセスしてください。

## テスト

各コンポーネントのテストを実行するには:

```bash
# 共通コードのテスト
cd common
cargo test

# フロントエンドのテスト
cd ../frontend
wasm-pack test --headless --chrome

# バックエンドのテスト
cd ../backend
cargo test
```

## ライセンス

MIT

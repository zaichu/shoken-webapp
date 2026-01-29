# 証券情報ウェブアプリケーション (shoken-webapp)

証券情報を検索・表示し、取引データを管理するためのウェブアプリケーションです。フロントエンドはReact+TypeScript、バックエンドはRust+Axumで構築されています。

## 機能概要

- 証券情報の検索と表示
- CSVからの取引データのインポートと分析
- 実現損益の計算と表示
- 各種証券情報サイトへのリンク生成
- Google OAuth認証（開発中）
- データの永続化と取引履歴管理（開発中）

## デモ

以下のリンクからデモアプリをご確認いただけます。

[デモサイト](https://zaichu.github.io/shoken-webapp/)

## アーキテクチャ

このプロジェクトは以下のコンポーネントで構成されています：

- **フロントエンド**: React + TypeScript + Vite
- **バックエンド**: Rust + Axum + SQLx + PostgreSQL
- **デプロイ**: Shuttle (バックエンド)、GitHub Pages (フロントエンド)

## 技術スタック

### フロントエンド
- **React** (18.2)
- **TypeScript** (5.4)
- **Vite**: 高速な開発環境とビルドツール
- **React Router**: SPAのルーティング
- **React Query**: データフェッチングとキャッシュ管理
- **Bootstrap**: UIコンポーネント
- **TailwindCSS**: ユーティリティファーストCSSフレームワーク
- **Axios**: HTTPクライアント

### バックエンド
- **Rust** (1.70以上)
- **Axum**: 高性能なWebフレームワーク
- **SQLx**: 非同期SQLツールキット
- **PostgreSQL**: データベース
- **Shuttle**: Rustアプリのデプロイプラットフォーム
- **OAuth2**: Google認証
- **Validator**: 入力データの検証

## 開発環境のセットアップ

### 必要なツール

- Rust (1.70以上)
- Node.js (18以上) と npm
- PostgreSQL (ローカル開発用)

### インストール手順

1. Rustとツールチェーンのインストール:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install cargo-shuttle  # Shuttleデプロイ用
```

2. リポジトリのクローン:

```bash
git clone https://github.com/yourusername/shoken-webapp.git
cd shoken-webapp
```

3. フロントエンドの依存関係をインストール:

```bash
cd frontend
npm install
```

### 環境変数の設定

バックエンドの`.env`ファイルを作成:

```bash
cd backend
cp .env.example .env  # .env.exampleがある場合
```

`.env`ファイルに以下の変数を設定:

```
DATABASE_URL=postgres://username:password@localhost:5432/shoken_db
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret
FRONTEND_URL=http://localhost:5173
```

フロントエンドの`.env.development.local`ファイルを作成:

```bash
cd frontend
cp .env.example .env.development.local  # .env.exampleがある場合
```

`.env.development.local`ファイルに以下の変数を設定:

```
VITE_API_URL=http://localhost:8000
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

# フロントエンド開発サーバーを実行
make run-frontend

# ビルド成果物をクリーン
make clean
```

### 手動ビルド手順

#### フロントエンドの開発サーバー起動

```bash
cd frontend
npm run dev
```

開発サーバーは http://localhost:5173 で起動します。

#### フロントエンドのビルド

```bash
cd frontend
npm run build
```

ビルド成果物は `frontend/dist` ディレクトリに生成されます。

#### バックエンドの実行

```bash
cd backend
cargo run
```

バックエンドサーバーは http://localhost:8000 で起動します。

## テスト実行

プロジェクト全体のテストを実行:

```bash
make test
```

個別にテストを実行:

```bash
# フロントエンドのテスト
cd frontend
npm test

# バックエンドのテスト
cd backend
cargo test
```

## データベースマイグレーション

SQLxを使用してデータベースマイグレーションを実行:

```bash
cd backend
cargo sqlx migrate run
```

新しいマイグレーションを作成:

```bash
cd backend
cargo sqlx migrate add <migration_name>
```

## デプロイ

### バックエンドのデプロイ (Shuttle)

```bash
make deploy-backend
# または
cd backend
cargo shuttle deploy
```

デプロイの状態を確認:

```bash
cd backend
cargo shuttle status
```

### フロントエンドのデプロイ (GitHub Pages)

GitHub Actionsを使用した自動デプロイが設定されています。mainブランチにプッシュすると、フロントエンドが自動的にビルドされてGitHub Pagesにデプロイされます。

手動でデプロイする場合:

```bash
cd frontend
npm run build
# 生成された dist ディレクトリの内容を GitHub Pages にデプロイ
```

## プロジェクト構造

```
shoken-webapp/
├── frontend/                # Reactフロントエンド
│   ├── src/                 # フロントエンドソースコード
│   │   ├── api/             # APIクライアント
│   │   ├── components/      # Reactコンポーネント
│   │   ├── hooks/           # カスタムフック
│   │   ├── pages/           # ページコンポーネント
│   │   ├── types/           # TypeScript型定義
│   │   └── utils/           # ユーティリティ関数
│   ├── public/              # 静的アセット
│   └── package.json         # フロントエンド依存関係
├── backend/                 # Axumバックエンド
│   ├── src/                 # バックエンドソースコード
│   │   ├── api/             # APIエンドポイント
│   │   ├── db/              # データベース操作
│   │   ├── models/          # データモデル
│   │   ├── services/        # ビジネスロジック
│   │   ├── utils/           # ユーティリティ関数
│   │   └── main.rs          # エントリーポイント
│   ├── migrations/          # SQLxデータベースマイグレーション
│   ├── tests/               # 統合テスト
│   └── Cargo.toml           # バックエンド依存関係
├── Cargo.toml               # ワークスペース設定
└── Makefile                 # ビルド/デプロイコマンド
```

## API仕様

バックエンドAPIは以下のエンドポイントを提供します:

### 認証

- `GET /auth/google`: Google OAuth認証の開始
- `GET /auth/google/callback`: Google OAuth認証のコールバック
- `GET /auth/me`: 現在のユーザー情報を取得
- `POST /auth/logout`: ログアウト
- `DELETE /auth/delete-account`: アカウント削除

### 証券情報

- `GET /stock/{query}`: 銘柄コードまたは名前で検索
- `POST /stock`: 銘柄情報を追加（**認証必須**）

### 配当金（認証必須）

- `GET /dividends`: 配当金一覧を取得
- `POST /dividends/bulk`: 配当金を一括追加
- `DELETE /dividends/all`: 配当金を全削除

### 国内株式（認証必須）

- `GET /domestic-stocks`: 国内株式一覧を取得
- `POST /domestic-stocks/bulk`: 国内株式を一括追加
- `DELETE /domestic-stocks/all`: 国内株式を全削除

### 投資信託（認証必須）

- `GET /mutualfunds`: 投資信託一覧を取得
- `POST /mutualfunds/bulk`: 投資信託を一括追加
- `DELETE /mutualfunds/all`: 投資信託を全削除

### 保有銘柄（認証必須）

- `GET /asset-balances`: 保有銘柄一覧を取得
- `POST /asset-balances/bulk`: 保有銘柄を一括追加（既存は更新）
- `DELETE /asset-balances/all`: 保有銘柄を全削除

### J-Quants API

- `GET /jquants/fins/statements`: 決算サマリーを取得

### ヘルスチェック

- `GET /health`: サーバー状態を確認

## テスト

本プロジェクトでは以下のテストを実装しています:

### バックエンドテスト

- 単体テスト: 各モジュールの機能をテスト
- 統合テスト: エンドポイントの挙動を検証
- モックを使用したサービスレイヤーのテスト

### フロントエンドテスト

- コンポーネントテスト: React Testing Libraryを使用
- ユーティリティ関数のテスト

## 貢献方法

プロジェクトへの貢献を歓迎します。以下の手順で貢献できます:

1. このリポジトリをフォーク
2. 新しいブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## ライセンス

MIT

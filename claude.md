# shoken-webapp プロジェクトガイド

## プロジェクト概要

shoken-webappは証券取引のためのウェブアプリケーションです。このプロジェクトはRustをバックエンドに使用し、フロントエンドもRustのYewフレームワークで構築されています。

## プロジェクト構造

```
shoken-webapp/
├── Makefile         # プロジェクト全体のMakefile
├── backend/         # バックエンドコード
│   ├── Makefile     # バックエンド専用のMakefile
│   ├── src/         # バックエンドのソースコード
│   └── ...
├── frontend/        # フロントエンドコード
│   ├── Makefile     # フロントエンド専用のMakefile
│   ├── src/         # フロントエンドのソースコード
│   └── ...
└── ...
```

## 開発ガイドライン

### Rust開発のベストプラクティス

#### コード構造

- `mod.rs`形式は使用せず、各ディレクトリには対応する名前の`.rs`ファイルを作成してください。
  ```
  ✅ 推奨: src/users/users.rs
  ❌ 非推奨: src/users/mod.rs
  ```

- 各モジュールは明確な責任を持つようにしてください。

#### テスト

- すべての機能には対応するテストを書いてください。
- テストは対象コードと同じファイル内に書くことを基本としてください。
- 複雑なテストケースは`tests`ディレクトリに分離しても構いません。

```rust
// テスト例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_user() {
        let mut repository = UserRepository::new();
        let user = User::new("test_user");
        repository.add(user.clone());
        assert_eq!(repository.find_by_name("test_user"), Some(user));
    }
}
```

#### エラー処理

- `Result<T, E>`と`Option<T>`を適切に使用してください。
- カスタムエラー型を定義して、エラー情報を明確に表現してください。

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("ユーザーが見つかりませんでした: {0}")]
    UserNotFound(String),
    
    #[error("データベースエラー: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("認証エラー")]
    AuthenticationError,
}

pub type AppResult<T> = Result<T, AppError>;
```

### フロントエンド開発のベストプラクティス (Yew)

- コンポーネントは小さく保ち、責任を明確にしてください。
- Yewのコンポーネントライフサイクルを適切に理解し活用してください。
- 状態管理にはYewのContextやAgentを使用してください。
- WebAssemblyのパフォーマンス特性を考慮したコード設計を心がけてください。

```rust
// Yewコンポーネント例
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub title: String,
    pub on_click: Callback<()>,
}

pub struct StockItem {
    props: Props,
}

impl Component for StockItem {
    type Message = ();
    type Properties = Props;
    
    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }
    
    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }
    
    fn view(&self) -> Html {
        html! {
            <div class="stock-item" onclick=self.props.on_click.reform(|_| ())>
                <h3>{ &self.props.title }</h3>
            </div>
        }
    }
}
```

## 環境構築

### 必要なツール

- Rust (最新の安定版)
- wasm-pack
- trunk (Yewアプリケーションのビルド用)
- shuttle-cli (バックエンドのデプロイ用)
- Docker
- Docker Compose

### 開発環境のセットアップ

```bash
# リポジトリのクローン
git clone https://github.com/your-org/shoken-webapp.git
cd shoken-webapp

# 必要なツールのインストール
cargo install wasm-pack
cargo install trunk
cargo install cargo-shuttle

# 初回ビルド (統合Makefileを使用)
make build
```

### 環境変数

`.env`ファイルを作成し、以下の環境変数を設定してください：

```
DATABASE_URL=postgres://postgres:password@localhost:5432/shoken_db
API_KEY=your_api_key_here
RUST_LOG=info
```

## Makefileの使用方法

このプロジェクトではMakefileを使用して開発作業を簡素化しています。ルートディレクトリ、backend、frontendの各ディレクトリにMakefileが用意されています。

### ルートディレクトリのMakefile

ルートディレクトリのMakefileは、フロントエンドとバックエンドの両方に対する操作を簡単に行うための統合インターフェースを提供します。

主なコマンド：
```bash
# 両方のプロジェクトをビルド
make build

# フロントエンドのみビルド
make build-frontend

# バックエンドのみビルド
make build-backend

# フロントエンドを実行
make run-frontend

# バックエンドを実行
make run-backend

# 両方のプロジェクトのクリーンアップ
make clean

# バックエンドのテスト実行
make test

# コード整形
make format

# リリースビルド
make release

# Shuttleにデプロイ
make deploy

# 利用可能なコマンドの一覧表示
make help
```

### バックエンドのMakefile

バックエンドディレクトリのMakefileは、バックエンド特有の操作を提供します。

主なコマンド：
```bash
# ローカルでShuttleプロジェクトを実行
cd backend && make run

# Shuttleにデプロイ
cd backend && make deploy

# ステータス確認
cd backend && make status

# ビルド
cd backend && make build

# テスト実行
cd backend && make test

# SQLxクエリキャッシュの準備
cd backend && make sqlx-prepare
```

### フロントエンドのMakefile

フロントエンドディレクトリのMakefileは、フロントエンド特有の操作を提供します。

主なコマンド：
```bash
# ビルド
cd frontend && make build

# 開発サーバーの実行
cd frontend && make run

# クリーンアップ
cd frontend && make clean
```

## ビルドと実行

### 統合コマンド（推奨）

```bash
# プロジェクト全体のビルド
make build

# フロントエンドの実行
make run-frontend

# バックエンドの実行
make run-backend
```

## テスト実行

```bash
# バックエンドのテスト
make test
```

## デプロイ

デプロイはShuttleを使用して行い、GitHub Actionsを通じて自動化されています。`main`ブランチへのプッシュによって、以下のステップが実行されます：

1. テストの実行
2. ビルド
3. Shuttleを使用したデプロイ

手動でデプロイする場合：
```bash
make deploy
```

## コントリビューションガイド

1. 新たな機能やバグ修正のためのブランチを作成してください (`feature/xxxx` または `fix/xxxx`)
2. コミットメッセージは明確に記述してください
3. プルリクエストを作成する前に、ローカルでテストを実行してください
4. コードレビューのフィードバックに基づいて修正してください

## API ドキュメント

APIドキュメントは自動生成され、開発サーバー起動時に以下のURLでアクセスできます：

- Swagger UI: http://localhost:8080/swagger-ui/
- OpenAPI JSON: http://localhost:8080/api-docs/

## トラブルシューティング

よくある問題と解決策：

1. データベース接続エラー
   - Docker Composeが正常に起動しているか確認してください
   - `.env`ファイルの`DATABASE_URL`が正しいか確認してください

2. WebAssemblyビルドエラー
   - wasm-packとtrunkが最新バージョンであることを確認してください
   - ブラウザのコンソールでエラーメッセージを確認してください

3. Shuttleデプロイエラー
   - `shuttle status`コマンドで現在のステータスを確認してください
   - `shuttle logs`コマンドでエラーログを確認してください

4. ビルドエラー
   - Rustのパッケージを最新にアップデートしてください
   - 依存関係の競合がないかチェックしてください
   - ビルドログを確認してください：`make build-log`

## 参考リソース

- [Rust公式ドキュメント](https://www.rust-lang.org/ja/documentation)
- [Yewフレームワーク](https://yew.rs/ja/)
- [WebAssembly公式サイト](https://webassembly.org/)
- [Shuttle ドキュメント](https://docs.shuttle.rs/)
- [Trunk ドキュメント](https://trunkrs.dev/)

# ベースイメージとして公式のRustイメージを使用します
FROM rust:latest

# アプリケーションディレクトリを作成して、Rustプロジェクトファイルをコピーします
WORKDIR /usr/src/app
COPY . .

# 必要な依存関係をインストールして、プロジェクトをリリースモードでビルドします
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

# 実行可能ファイルをエントリーポイントとして設定します
CMD ["./target/release/shoken-webapp"]

# ビルドステージ
FROM rust:1.96-bookworm AS builder

WORKDIR /app

# 依存関係のキャッシュ用にCargo.tomlとCargo.lockをコピー
COPY Cargo.toml Cargo.lock ./

# ダミーのsrc/main.rsを作成して依存関係をキャッシュ
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 実際のソースコードをコピー
COPY src ./src
COPY migrations ./migrations

# 依存関係のビルドキャッシュを無効化しないために、全 Rust ソースのタイムスタンプを更新
RUN find src -name "*.rs" -exec touch {} +

# SQLxのオフラインモード用（.sqlxディレクトリがある場合）
# 注意: .sqlxディレクトリが存在しない場合はこの行をコメントアウトするか、
# cargo sqlx prepare を実行して作成してください
# COPY .sqlx ./.sqlx

# 本番ビルド（SQLX_OFFLINE=falseでビルド - マクロはランタイムでチェックされる）
RUN cargo build --release

# 実行ステージ
FROM debian:bookworm-slim

WORKDIR /app

# 必要なランタイムライブラリをインストール
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# ビルド成果物をコピー
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/migrations /app/migrations

# 非rootユーザーで実行
RUN useradd -m appuser
USER appuser

# ポートを公開
EXPOSE 8080

# 環境変数のデフォルト値
ENV PORT=8080
ENV RUST_LOG=info

# アプリケーションを実行
CMD ["/app/backend"]

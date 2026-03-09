# shoken-webapp backend Makefile

# 変数
CARGO := cargo
FLY := fly

# デフォルトターゲット
.PHONY: all
all: check run

# 基本ターゲット
.PHONY: run
run:
	@echo "🚀 ローカルでサーバーを起動します..."
	$(CARGO) run --bin backend

.PHONY: deploy
deploy:
	@echo "🚀 Fly.ioにデプロイします..."
	$(FLY) deploy

.PHONY: status
status:
	@echo "📊 Fly.ioサービスのステータスを確認します..."
	$(FLY) status

.PHONY: logs
logs:
	@echo "📜 アプリケーションログを表示します..."
	$(FLY) logs

# ビルド関連
.PHONY: build
build:
	@echo "🔨 バックエンドをビルドします..."
	$(CARGO) build

.PHONY: build-release
build-release:
	@echo "🔨 リリースビルドを作成します..."
	$(CARGO) build --release

.PHONY: build-log
build-log:
	@echo "🔨 バックエンドをビルドし、ログを保存します..."
	-$(CARGO) build > build.log 2>&1 || echo "ビルドに失敗しました: 終了コード $$?" >> build.log
	@echo "ビルド出力は build.log に保存されました"

.PHONY: check
check:
	@echo "🔍 コンパイルチェックを実行します..."
	$(CARGO) check

# テスト関連
.PHONY: test
test:
	@echo "🧪 テストを実行します..."
	$(CARGO) test

# データベース関連
.PHONY: migrate
migrate:
	@echo "📦 データベースマイグレーションを実行します..."
	$(CARGO) sqlx migrate run

.PHONY: migrate-local
migrate-local:
	@echo "📦 ローカルDocker DBにマイグレーションを実行します..."
	DATABASE_URL=postgresql://user:password@localhost:5432/shoken_db $(CARGO) sqlx migrate run

.PHONY: repair-and-migrate-local
repair-and-migrate-local:
	@echo "🔧 ローカルDocker DB を旧 0001〜0017 から新 0001〜0009 に切り替えます..."
	@echo "  (version=17 未達の場合はエラーで停止します)"
	DATABASE_URL=postgresql://user:password@localhost:5432/shoken_db psql postgresql://user:password@localhost:5432/shoken_db -f scripts/repair-migrations.sql
	DATABASE_URL=postgresql://user:password@localhost:5432/shoken_db $(CARGO) sqlx migrate run

.PHONY: sqlx-prepare
sqlx-prepare:
	@echo "📦 SQLxクエリキャッシュを準備します..."
	$(CARGO) sqlx prepare --merged

# Docker関連
.PHONY: docker-build
docker-build:
	@echo "🐳 Dockerイメージをビルドします..."
	docker build -t shoken-backend .

.PHONY: db-up
db-up:
	@echo "🐳 PostgreSQL を Docker で起動します..."
	docker compose -f docker-compose.yml up -d

.PHONY: db-down
db-down:
	@echo "🐳 PostgreSQL を停止します..."
	docker compose -f docker-compose.yml down

.PHONY: db-logs
db-logs:
	@echo "🐳 PostgreSQL のログを表示します..."
	docker compose -f docker-compose.yml logs -f postgres

# ヘルプ
.PHONY: help
help:
	@echo "shoken-webapp バックエンド Makefile"
	@echo ""
	@echo "利用可能なターゲット:"
	@echo "  all           : デフォルトターゲット、チェックして実行します"
	@echo "  run           : ローカルでサーバーを起動します"
	@echo "  deploy        : Fly.ioにデプロイします"
	@echo "  status        : Fly.ioサービスのステータスを確認します"
	@echo "  logs          : アプリケーションログを表示します"
	@echo "  build         : バックエンドをビルドします"
	@echo "  build-release : リリースビルドを作成します"
	@echo "  build-log     : バックエンドをビルドし、結果をbuild.logに保存します"
	@echo "  check         : コンパイルチェックのみを実行します"
	@echo "  test          : テストを実行します"
	@echo "  migrate       : データベースマイグレーションを実行します"
	@echo "  migrate-local           : ローカルDocker DBにマイグレーションを実行します"
	@echo "  repair-and-migrate-local: 旧 0001〜0017 → 新 0001〜0009 への一回限り切り替え"
	@echo "  sqlx-prepare            : SQLxクエリキャッシュを準備します"
	@echo "  docker-build  : Dockerイメージをビルドします"
	@echo "  db-up         : PostgreSQL を Docker で起動します"
	@echo "  db-down       : PostgreSQL を停止します"
	@echo "  db-logs       : PostgreSQL のログを表示します"
	@echo "  clean         : ビルド成果物をクリーンアップします"
	@echo "  clean-build   : クリーンアップして再ビルドします"
	@echo "  help          : このヘルプを表示します"

# ビルドとクリーンアップ
.PHONY: clean-build
clean-build:
	@echo "🚧 ビルドをクリーンアップして再ビルドします..."
	@rm -rf target
	$(CARGO) clean
	$(CARGO) build --verbose

# クリーン
.PHONY: clean
clean:
	@echo "🧹 ビルド成果物をクリーンアップします..."
	$(CARGO) clean

# shoken-webapp backend Makefile

# 変数
SHUTTLE := shuttle
CARGO := cargo

# デフォルトターゲット
.PHONY: all
all: check run

# 基本ターゲット
.PHONY: run
run:
	@echo "🚀 ローカルでShuttleプロジェクトを実行します..."
	$(SHUTTLE) run

.PHONY: deploy
deploy:
	@echo "🚀 Shuttleにプロジェクトをデプロイします..."
	$(SHUTTLE) deploy

.PHONY: deploy-dirty
deploy-dirty:
	@echo "🚀 未コミットの変更でShuttleにデプロイします..."
	$(SHUTTLE) deploy --allow-dirty

.PHONY: status
status:
	@echo "📊 Shuttleサービスのステータスを確認します..."
	$(SHUTTLE) status

.PHONY: stop
stop:
	@echo "🛑 Shuttleサービスを停止します..."
	$(SHUTTLE) stop

.PHONY: logs
logs:
	@echo "📜 ビルドとデプロイメントのログを表示します..."
	$(SHUTTLE) logs

# ビルド関連
.PHONY: build
build:
	@echo "🔨 バックエンドをビルドします..."
	$(CARGO) build

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

# SQLx関連
.PHONY: sqlx-prepare
sqlx-prepare:
	@echo "📦 SQLxクエリキャッシュを準備します..."
	$(CARGO) sqlx prepare --merged

# ヘルプ
.PHONY: help
help:
	@echo "shoken-webapp バックエンド Makefile"
	@echo ""
	@echo "利用可能なターゲット:"
	@echo "  all          : デフォルトターゲット、チェックして実行します"
	@echo "  run          : ローカルでShuttleプロジェクトを実行します"
	@echo "  deploy       : Shuttleにプロジェクトをデプロイします"
	@echo "  deploy-dirty : 未コミットの変更でShuttleにデプロイします"
	@echo "  status       : Shuttleサービスのステータスを確認します"
	@echo "  stop         : Shuttleサービスを停止します"
	@echo "  logs         : ビルドとデプロイメントのログを表示します"
	@echo "  build        : バックエンドをビルドします"
	@echo "  build-log    : バックエンドをビルドし、結果をbuild.logに保存します"
	@echo "  check        : コンパイルチェックのみを実行します"
	@echo "  test         : テストを実行します"
	@echo "  sqlx-prepare : SQLxクエリキャッシュを準備します"
	@echo "  help         : このヘルプを表示します"

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
	$(SHUTTLE) clean
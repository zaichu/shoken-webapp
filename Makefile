# shoken-webapp 統合 Makefile

.PHONY: all build build-log run clean test format help dev deploy

# プロジェクトのルートパス
ROOT_DIR := $(shell pwd)
BACKEND_DIR := $(ROOT_DIR)/backend
FRONTEND_DIR := $(ROOT_DIR)/frontend

# デフォルトターゲット - 両方をクリーンしてからビルド
all: clean build

# ビルド - 両方のプロジェクトをビルド（サブプロジェクトのMakefileを使用）
build:
	@echo "Building backend..."
	-@cd $(BACKEND_DIR) && make build
	@echo "Building frontend..."
	-@cd $(FRONTEND_DIR) && make build

# ビルドとログ出力
build-log:
	@echo "Building backend with log output..."
	-@cd $(BACKEND_DIR) && make build-log
	@echo "Building frontend with log output..."
	-@cd $(FRONTEND_DIR) && make build-log
	@echo "Build logs have been generated for both frontend and backend"

# フロントエンドのビルド
build-frontend:
	@echo "Building frontend..."
	-@cd $(FRONTEND_DIR) && make build

# バックエンドのビルド
build-backend:
	@echo "Building backend..."
	-@cd $(BACKEND_DIR) && make build

# フロントエンドの実行
run-frontend:
	@echo "Running frontend..."
	-@cd $(FRONTEND_DIR) && make run

# バックエンドの実行
run-backend:
	@echo "Running backend server..."
	-@cd $(BACKEND_DIR) && make run

# 全体クリーン
clean:
	@echo "Cleaning frontend..."
	-@cd $(FRONTEND_DIR) && make clean
	@echo "Cleaning backend..."
	-@cd $(BACKEND_DIR) && make clean
	@echo "Cleaned both frontend and backend"

# テスト
test:
	@echo "Testing backend..."
	-@cd $(BACKEND_DIR) && make test
	@echo "All tests completed"

# コード整形
format:
	@echo "Formatting backend code..."
	-@cd $(BACKEND_DIR) && make format
	@echo "Formatting completed"

# リリースビルド
release:
	@echo "Building frontend release..."
	-@cd $(FRONTEND_DIR) && make build
	@echo "Building backend release..."
	-@cd $(BACKEND_DIR) && make build-release
	@echo "Release builds completed"

# リリースビルド（ログ出力）
release-log:
	@echo "Building release with logs..."
	-@cd $(FRONTEND_DIR) && make build-log
	-@cd $(BACKEND_DIR) && make build-release-log
	@echo "Release builds completed and logs saved"

# Shuttle関連コマンド
deploy:
	@echo "Deploying application to Shuttle..."
	-@cd $(BACKEND_DIR) && make shuttle-deploy
	@echo "Deployment completed"

deploy-dirty:
	@echo "Deploying application with uncommitted changes..."
	-@cd $(BACKEND_DIR) && make shuttle-deploy-dirty
	@echo "Deployment completed"

status:
	@echo "Checking deployment status..."
	-@cd $(BACKEND_DIR) && make shuttle-status

# ヘルプ
help:
	@echo "shoken-webapp 統合 Makefile"
	@echo ""
	@echo "利用可能なターゲット:"
	@echo "  all             : 両方のプロジェクトをビルド（デフォルト）"
	@echo "  build           : フロントエンドとバックエンドの両方をビルド"
	@echo "  build-log       : 両方をビルドし、ログを出力"
	@echo "  build-frontend  : フロントエンドのみビルド"
	@echo "  build-backend   : バックエンドのみビルド"
	@echo "  run-frontend    : フロントエンドを実行"
	@echo "  run-backend     : バックエンドサーバーを実行"
	@echo "  clean           : 両方のプロジェクトのビルド成果物をクリーン"
	@echo "  test            : バックエンドのテストを実行"
	@echo "  format          : バックエンドのコードをフォーマット"
	@echo "  release         : リリースビルドを実行"
	@echo "  release-log     : リリースビルドを実行し、ログを出力"
	@echo "  deploy          : アプリケーションをShuttleにデプロイ"
	@echo "  deploy-dirty    : 未コミットの変更でアプリケーションをデプロイ"
	@echo "  status          : デプロイメントステータスを確認"
	@echo "  help            : このヘルプを表示"

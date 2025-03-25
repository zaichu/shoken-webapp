# shoken-webapp backend Makefile

# デフォルトターゲット
.PHONY: all
all: build

# ビルド関連変数
CARGO := cargo
CARGO_FLAGS :=
RELEASE_FLAGS := --release
DEBUG_FLAGS :=
SHUTTLE := shuttle

# ビルドターゲット
.PHONY: build
build:
	@echo "Building backend..."
	@$(CARGO) build $(CARGO_FLAGS)

.PHONY: build-release
build-release:
	@echo "Building release version..."
	@$(CARGO) build $(RELEASE_FLAGS)

# テスト関連
.PHONY: test
test:
	@echo "Running tests..."
	@$(CARGO) test

.PHONY: test-verbose
test-verbose:
	@echo "Running tests with verbose output..."
	@$(CARGO) test -- --nocapture

# リントとフォーマット
.PHONY: lint
lint:
	@echo "Running clippy..."
	@$(CARGO) clippy

.PHONY: format
format:
	@echo "Formatting code..."
	@$(CARGO) fmt

# クリーン
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean

# 実行関連
.PHONY: run
run:
	@echo "Running backend..."
	@$(CARGO) run

.PHONY: run-release
run-release:
	@echo "Running release build..."
	@$(CARGO) run --release

# Shuttle関連
.PHONY: shuttle-deploy
shuttle-deploy:
	@echo "Deploying to Shuttle..."
	@$(SHUTTLE) deploy

.PHONY: shuttle-deploy-dirty
shuttle-deploy-dirty:
	@echo "Deploying to Shuttle with uncommitted changes..."
	@$(SHUTTLE) deploy --allow-dirty

.PHONY: shuttle-status
shuttle-status:
	@echo "Checking Shuttle project status..."
	@$(SHUTTLE) project status

.PHONY: shuttle-local
shuttle-local:
	@echo "Running locally with Shuttle runtime..."
	@$(SHUTTLE) run

# ヘルプ
.PHONY: help
help:
	@echo "shoken-webapp backend Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all             : Default target, builds the backend"
	@echo "  build           : Build backend in debug mode"
	@echo "  build-release   : Build backend in release mode"
	@echo "  test            : Run tests"
	@echo "  test-verbose    : Run tests with verbose output"
	@echo "  lint            : Run clippy for linting"
	@echo "  format          : Format code with rustfmt"
	@echo "  clean           : Clean build artifacts"
	@echo "  run             : Run backend in debug mode"
	@echo "  run-release     : Run backend in release mode"
	@echo "  shuttle-deploy  : Deploy to Shuttle"
	@echo "  shuttle-deploy-dirty : Deploy to Shuttle with uncommitted changes"
	@echo "  shuttle-status  : Check Shuttle project status"
	@echo "  shuttle-local   : Run locally with Shuttle runtime"
	@echo "  help            : Show this help"
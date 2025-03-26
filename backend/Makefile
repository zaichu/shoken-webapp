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

.PHONY: build-log
build-log:
	@echo "Building backend and saving output to build.log..."
	@$(CARGO) build $(CARGO_FLAGS) > build.log 2>&1
	@echo "Build output saved to build.log"

.PHONY: build-release
build-release:
	@echo "Building release version..."
	@$(CARGO) build $(RELEASE_FLAGS)

.PHONY: build-release-log
build-release-log:
	@echo "Building release version and saving output to build-release.log..."
	@$(CARGO) build $(RELEASE_FLAGS) > build-release.log 2>&1
	@echo "Build output saved to build-release.log"

# テスト関連
.PHONY: test
test:
	@echo "Running tests..."
	@$(CARGO) test

.PHONY: test-log
test-log:
	@echo "Running tests and saving output to test.log..."
	@$(CARGO) test > test.log 2>&1
	@echo "Test output saved to test.log"

.PHONY: test-verbose
test-verbose:
	@echo "Running tests with verbose output..."
	@$(CARGO) test -- --nocapture

.PHONY: test-verbose-log
test-verbose-log:
	@echo "Running tests with verbose output and saving to test-verbose.log..."
	@$(CARGO) test -- --nocapture > test-verbose.log 2>&1
	@echo "Verbose test output saved to test-verbose.log"

# リントとフォーマット
.PHONY: lint
lint:
	@echo "Running clippy..."
	@$(CARGO) clippy

.PHONY: lint-log
lint-log:
	@echo "Running clippy and saving output to lint.log..."
	@$(CARGO) clippy > lint.log 2>&1
	@echo "Lint output saved to lint.log"

.PHONY: format
format:
	@echo "Formatting code..."
	@$(CARGO) fmt

.PHONY: format-log
format-log:
	@echo "Formatting code and saving output to format.log..."
	@$(CARGO) fmt > format.log 2>&1
	@echo "Format output saved to format.log"

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

.PHONY: run-log
run-log:
	@echo "Running backend and saving output to run.log..."
	@$(CARGO) run > run.log 2>&1
	@echo "Run output saved to run.log"

.PHONY: run-release
run-release:
	@echo "Running release build..."
	@$(CARGO) run --release

.PHONY: run-release-log
run-release-log:
	@echo "Running release build and saving output to run-release.log..."
	@$(CARGO) run --release > run-release.log 2>&1
	@echo "Release run output saved to run-release.log"

# Shuttle関連
.PHONY: shuttle-deploy
shuttle-deploy:
	@echo "Deploying to Shuttle..."
	@$(SHUTTLE) deploy

.PHONY: shuttle-deploy-log
shuttle-deploy-log:
	@echo "Deploying to Shuttle and saving output to shuttle-deploy.log..."
	@$(SHUTTLE) deploy > shuttle-deploy.log 2>&1
	@echo "Shuttle deploy output saved to shuttle-deploy.log"

.PHONY: shuttle-deploy-dirty
shuttle-deploy-dirty:
	@echo "Deploying to Shuttle with uncommitted changes..."
	@$(SHUTTLE) deploy --allow-dirty

.PHONY: shuttle-deploy-dirty-log
shuttle-deploy-dirty-log:
	@echo "Deploying to Shuttle with uncommitted changes and saving output to shuttle-deploy-dirty.log..."
	@$(SHUTTLE) deploy --allow-dirty > shuttle-deploy-dirty.log 2>&1
	@echo "Shuttle dirty deploy output saved to shuttle-deploy-dirty.log"

.PHONY: shuttle-status
shuttle-status:
	@echo "Checking Shuttle project status..."
	@$(SHUTTLE) project status

.PHONY: shuttle-status-log
shuttle-status-log:
	@echo "Checking Shuttle project status and saving output to shuttle-status.log..."
	@$(SHUTTLE) project status > shuttle-status.log 2>&1
	@echo "Shuttle status output saved to shuttle-status.log"

.PHONY: shuttle-local
shuttle-local:
	@echo "Running locally with Shuttle runtime..."
	@$(SHUTTLE) run

.PHONY: shuttle-local-log
shuttle-local-log:
	@echo "Running locally with Shuttle runtime and saving output to shuttle-local.log..."
	@$(SHUTTLE) run > shuttle-local.log 2>&1
	@echo "Shuttle local run output saved to shuttle-local.log"

# ヘルプ
.PHONY: help
help:
	@echo "shoken-webapp backend Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all             : Default target, builds the backend"
	@echo "  build           : Build backend in debug mode"
	@echo "  build-log       : Build backend and save output to build.log"
	@echo "  build-release   : Build backend in release mode"
	@echo "  build-release-log: Build backend in release mode and save output to build-release.log"
	@echo "  test            : Run tests"
	@echo "  test-log        : Run tests and save output to test.log"
	@echo "  test-verbose    : Run tests with verbose output"
	@echo "  test-verbose-log: Run tests with verbose output and save to test-verbose.log"
	@echo "  lint            : Run clippy for linting"
	@echo "  lint-log        : Run clippy and save output to lint.log"
	@echo "  format          : Format code with rustfmt"
	@echo "  format-log      : Format code and save output to format.log"
	@echo "  clean           : Clean build artifacts"
	@echo "  run             : Run backend in debug mode"
	@echo "  run-log         : Run backend and save output to run.log"
	@echo "  run-release     : Run backend in release mode"
	@echo "  run-release-log : Run backend in release mode and save output to run-release.log"
	@echo "  shuttle-deploy  : Deploy to Shuttle"
	@echo "  shuttle-deploy-log: Deploy to Shuttle and save output to shuttle-deploy.log"
	@echo "  shuttle-deploy-dirty : Deploy to Shuttle with uncommitted changes"
	@echo "  shuttle-deploy-dirty-log: Deploy to Shuttle with uncommitted changes and save output"
	@echo "  shuttle-status  : Check Shuttle project status"
	@echo "  shuttle-status-log: Check Shuttle project status and save output to shuttle-status.log"
	@echo "  shuttle-local   : Run locally with Shuttle runtime"
	@echo "  shuttle-local-log: Run locally with Shuttle runtime and save output to shuttle-local.log"
	@echo "  help            : Show this help"
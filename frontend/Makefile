.PHONY: all build build-log run run-log clean clean-log help

all: clean build run

build:
	trunk build --release --public-url /shoken-webapp-wasm/

build-log:
	trunk build --release --public-url /shoken-webapp-wasm/ > build.log 2>&1
	@echo "Build output saved to build.log"

run:
	trunk serve --release --public-url /shoken-webapp-wasm/

run-log:
	trunk serve --release --public-url /shoken-webapp-wasm/ > run.log 2>&1
	@echo "Run output saved to run.log"

clean:
	rm -rf dist
	cargo clean

clean-log:
	(rm -rf dist && cargo clean) > clean.log 2>&1
	@echo "Clean output saved to clean.log"

# ヘルプ
help:
	@echo "shoken-webapp frontend Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all        : Default target, cleans, builds and runs the frontend"
	@echo "  build      : Build frontend in release mode"
	@echo "  build-log  : Build frontend and save output to build.log"
	@echo "  run        : Run frontend development server"
	@echo "  run-log    : Run frontend development server and save output to run.log"
	@echo "  clean      : Clean build artifacts"
	@echo "  clean-log  : Clean build artifacts and save output to clean.log"
	@echo "  help       : Show this help"

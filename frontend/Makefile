.PHONY: all install build dev clean help

all: clean install build

install:
	npm install

build:
	npm run build

dev:
	npm run dev

clean:
	rm -rf node_modules dist

# ヘルプ
help:
	@echo "shoken-webapp frontend Makefile (React)"
	@echo ""
	@echo "Available targets:"
	@echo "  all        : Default target, cleans, installs dependencies, and builds the frontend"
	@echo "  install    : Install npm dependencies"
	@echo "  build      : Build frontend in production mode"
	@echo "  dev        : Run development server"
	@echo "  clean      : Clean build artifacts and dependencies"
	@echo "  help       : Show this help"

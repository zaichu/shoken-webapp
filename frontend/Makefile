.PHONY: all build run clean

all: clean build run

build:
	trunk build --release --public-url /shoken-webapp-wasm/

run:
	trunk serve --release --public-url /shoken-webapp-wasm/

clean:
	rm -rf dist
	cargo clean

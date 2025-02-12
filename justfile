set dotenv-required
install_target := env("INSTALL_TARGET")

default: install

dev:
	cargo fmt
	cargo run

build:
	cargo build --release

install: build
	cp -v target/release/watcher {{install_target}}

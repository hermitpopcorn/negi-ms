set dotenv-required
install_target_watcher := env("INSTALL_TARGET_WATCHER")
install_target_duplifinder := env("INSTALL_TARGET_DUPLIFINDER")

default: install

format:
  cargo fmt

devw: format
	cargo run --bin watcher

devd: format
	cargo run --bin duplifinder

build:
	cargo build --release

install: build
	cp -v target/release/watcher {{install_target_watcher}}
	cp -v target/release/duplifinder {{install_target_duplifinder}}

set dotenv-required
install_target_watcher := env("INSTALL_TARGET_WATCHER")
install_target_marksman := env("INSTALL_TARGET_MARKSMAN")
install_target_clerk := env("INSTALL_TARGET_CLERK")

default: install

format:
	cargo fmt

devw: format
	cargo run --bin watcher

devd: format
	cargo run --bin marksman

devc: format
	cargo run --bin clerk

build: format
	cargo build --release
	cd clerk-fe && pnpm run build

install: build
	cp -v target/release/watcher {{install_target_watcher}}
	cp -v target/release/marksman {{install_target_marksman}}
	cp -v target/release/clerk {{install_target_clerk}}
	CLERK_TARGET_DIR=$(dirname {{install_target_clerk}})/clerk-fe-public; rm -r $CLERK_TARGET_DIR && cp -r clerk-fe/dist $CLERK_TARGET_DIR

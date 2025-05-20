build:
	@cargo clean
	@cargo build --release
	@mkdir -p dist
	@cp target/release/load dist
	@cp target/release/chat dist

test:
	@cargo test

fix:
	@cargo +nightly fmt
	@cargo fix --allow-dirty --allow-staged
	@cargo clippy --fix --allow-dirty --allow-staged
	@cargo test

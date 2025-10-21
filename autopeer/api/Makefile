.PHONY: build check clean run test

build:
	cargo build --release

check:
	cargo fmt
	cargo clippy -- -D warnings

clean:
	cargo clean

run:
	cargo run

test: check
	cargo test
all: build

build:
	cargo build

run:
	cargo run

test:
	cargo test -- --nocapture

lint:
	cargy clippy -- -D warnings

clean:
	cargo clean

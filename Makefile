# Makefile for NextShell project
.PHONY: dev ci deploy

dev:
	# Run the default 'hello' example in server
	cargo run --manifest-path server/Cargo.toml --example hello

ci:
	# Style checks, tests, and docs
	cargo fmt --all -- --check
	cargo clippy --all -- -D warnings
	cargo test --workspace
	cargo rustdoc -- -D broken_intra_doc_links

deploy:
	# Build and test in release mode
	cargo build --workspace --release
	cargo test --workspace --release
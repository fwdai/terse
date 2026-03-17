.PHONY: test test-ts test-rust

test: test-ts test-rust

test-ts:
	cd packages/terse && bun test

test-rust:
	cd crates/terse && cargo test

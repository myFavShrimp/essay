.PHONY: test

test:
	cargo test
	cargo test --example duckdb_sync
	cargo test --example sqlx_sqlite

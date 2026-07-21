# Migration
migration-update:
	cargo run --bin migrate -- migration generate

migration-apply:
	cargo run --bin migrate -- migration apply
.PHONY: fmt fmt-check clippy test check run run-dev

MUSIC_BACKEND ?= rodio
MUSIC_ENABLED ?= true

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

check: fmt-check clippy test

run:
	SPACEFLIGHTS_MUSIC_ENABLED=$(MUSIC_ENABLED) \
	SPACEFLIGHTS_MUSIC_BACKEND=$(MUSIC_BACKEND) \
	cargo run -p spaceflights-app --bin spaceflights-app

run-dev:
	SPACEFLIGHTS_MUSIC_ENABLED=$(MUSIC_ENABLED) \
	SPACEFLIGHTS_MUSIC_BACKEND=$(MUSIC_BACKEND) \
	cargo run -p spaceflights-app --bin spaceflights-dev-debug

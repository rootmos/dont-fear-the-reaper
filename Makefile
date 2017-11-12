TARGET=target/debug
REAPER=$(TARGET)/reaper
DAEMON=$(TARGET)/daemon

.PHONY: test
test: build
	REAPER=$(REAPER) DAEMON=$(DAEMON) RUST_LOG=debug ./test.sh

.PHONY: build
build:
	cargo build --bins

TARGET=target/debug
REAPER=$(TARGET)/reaper
DAEMON=$(TARGET)/examples/example-daemon

.PHONY: test
test: build
	REAPER=$(REAPER) DAEMON=$(DAEMON) RUST_LOG=debug ./test.sh

.PHONY: build
build:
	cargo build --bins --examples

CARGO ?= cargo
PACKAGE := aiqiezi-gui
BUILD_DATE ?= $(shell date -u +%Y%m%d%H%M%S)
DIST_DIR ?= dist
OS ?= $(shell uname -s)
EXE_EXT ?= $(if $(filter Windows_NT,$(OS)),.exe,)
CLI_GIT ?= https://github.com/gocpicnic/aiqiezi-cli
CLI_REV := $(shell sed -n 's/.*rev = "\([a-f0-9][a-f0-9]*\)".*/\1/p' Cargo.toml | head -n 1)
CLI_ROOT := target/cli-bundle
AIQIEZI_CLI_BIN ?=
ARGS ?=

.DEFAULT_GOAL := help

.PHONY: help build build-gui bundle-cli check-cli-rev run fmt fmt-check check test clippy ci clean

help:
	@printf 'Usage:\n'
	@printf '  make build        Build GUI and bundle the aiqiezi daemon into %s/\n' "$(DIST_DIR)"
	@printf '  make build-gui    Build only the GUI binary into %s/\n' "$(DIST_DIR)"
	@printf '  make bundle-cli   Build/copy only the aiqiezi daemon into %s/\n' "$(DIST_DIR)"
	@printf '  make run          Run the GUI from source\n'
	@printf '  make fmt          Format all Rust code\n'
	@printf '  make fmt-check    Check Rust formatting\n'
	@printf '  make check        Run cargo check\n'
	@printf '  make test         Run all tests\n'
	@printf '  make clippy       Run clippy with warnings denied\n'
	@printf '  make ci           Run all checks used by CI\n'
	@printf '  make clean        Remove build artifacts and %s\n' "$(DIST_DIR)"
	@printf '\n'
	@printf 'Optional: AIQIEZI_CLI_BIN=/path/to/aiqiezi make build\n'

build: build-gui bundle-cli

build-gui:
	BUILD_DATE="$(BUILD_DATE)" $(CARGO) build --release --locked
	mkdir -p "$(DIST_DIR)"
	cp "target/release/aiqiezi-gui$(EXE_EXT)" "$(DIST_DIR)/aiqiezi-gui$(EXE_EXT)"
	cp LICENSE "$(DIST_DIR)/LICENSE"

bundle-cli: check-cli-rev
	mkdir -p "$(DIST_DIR)"
	@if [ -n "$(AIQIEZI_CLI_BIN)" ]; then \
		cp "$(AIQIEZI_CLI_BIN)" "$(DIST_DIR)/aiqiezi$(EXE_EXT)"; \
	else \
		BUILD_DATE="$(BUILD_DATE)" $(CARGO) install \
			--git "$(CLI_GIT)" \
			--rev "$(CLI_REV)" \
			--bin aiqiezi \
			--root "$(CLI_ROOT)" \
			--force \
			--locked; \
		cp "$(CLI_ROOT)/bin/aiqiezi$(EXE_EXT)" "$(DIST_DIR)/aiqiezi$(EXE_EXT)"; \
	fi
	cp LICENSE "$(DIST_DIR)/LICENSE"

check-cli-rev:
	@test -n "$(CLI_REV)" || { \
		echo "error: could not read the aiqiezi-client Git revision from Cargo.toml" >&2; \
		exit 1; \
	}

run:
	BUILD_DATE="$(BUILD_DATE)" $(CARGO) run --locked -- $(ARGS)

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

check:
	$(CARGO) check --locked

test:
	$(CARGO) test --locked

clippy:
	$(CARGO) clippy --all-targets --locked -- -D warnings

ci: fmt-check check test clippy

clean:
	$(CARGO) clean
	rm -rf "$(DIST_DIR)"

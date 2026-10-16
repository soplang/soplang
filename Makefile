# Makefile for Soplang - The Somali Programming Language
# This file provides simple commands for common development tasks

# Default target
.PHONY: help
help:
	@echo "Soplang Development Commands:"
	@echo "  make build     - Build release binary (cargo build --release)"
	@echo "  make run       - Run a .sop file: make run FILE=examples/01_hello.sop"
	@echo "  make shell     - Run interactive REPL"
	@echo "  make test      - Run unit and integration tests"
	@echo "  make bench     - Run criterion benchmarks"
	@echo "  make precommit - Run pre-commit hooks on all files"
	@echo "  make clean     - Remove build artifacts"
	@echo "  make docker-build - Build Docker image"
	@echo "  make docker-run  - Run Soplang in Docker container"

# Pre-commit hooks
.PHONY: precommit
precommit:
	pre-commit run --all-files

# Clean up target
.PHONY: clean
clean:
	cargo clean 2>/dev/null || true

# Docker targets
.PHONY: docker-build docker-run
docker-build:
	docker-compose build

docker-run:
	docker-compose up -d
	docker-compose exec soplang ./target/release/soplang -i

# --- Rust targets ---
.PHONY: build run shell test bench
build:
	cargo build --release

run:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make run FILE=<filename>"; \
		exit 1; \
	fi
	./target/release/soplang $(FILE)

shell:
	./target/release/soplang -i

test:
	cargo test

bench:
	cargo bench

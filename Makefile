# AstraWeave Gaming Engine - Makefile
# Provides convenient targets for common development tasks

.PHONY: help setup build build-all test clean format lint audit example status check

# Default target
help:
	@echo "AstraWeave Gaming Engine - Available Commands:"
	@echo ""
	@echo "Setup:"
	@echo "  setup       - Set up development environment (runs bootstrap script)"
	@echo "  validate    - Validate current environment setup"
	@echo ""
	@echo "Building:"
	@echo "  build       - Build core components (quick)"
	@echo "  build-all   - Build all working components"
	@echo "  clean       - Clean build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  test        - Run tests for all working components"
	@echo "  format      - Format code with rustfmt"
	@echo "  lint        - Run clippy linter"
	@echo "  audit       - Run security audit"
	@echo "  check       - Run comprehensive development checks"
	@echo ""
	@echo "Examples:"
	@echo "  example     - Run hello_companion example"
	@echo ""
	@echo "Utilities:"
	@echo "  status      - Show project status"
	@echo "  update      - Update dependencies"
	@echo ""
	@echo "For more options, use: ./scripts/dev.sh help"

# Setup and validation
setup:
	@echo "Setting up development environment..."
	@chmod +x scripts/bootstrap.sh scripts/dev.sh
	@./scripts/bootstrap.sh

validate:
	@./scripts/dev.sh validate

# Building
build:
	@./scripts/dev.sh build

build-all:
	@./scripts/dev.sh build-all

clean:
	@./scripts/dev.sh clean

# Development workflow
test:
	@./scripts/dev.sh test

format:
	@cargo fmt --all

lint:
	@./scripts/dev.sh quality

audit:
	@./scripts/dev.sh audit

check:
	@./scripts/dev.sh check

# Examples and utilities
example:
	@./scripts/dev.sh example

status:
	@./scripts/dev.sh status

update:
	@./scripts/dev.sh update

# Cargo aliases (for convenience)
build-core:
	@cargo build-core

build-working:
	@cargo build-working

check-all:
	@cargo check-all

test-all:
	@cargo test-all

clippy-all:
	@cargo clippy-all

# Development workflow shortcuts
dev: format lint test
	@echo "Development checks completed"

ci: clean build-all test lint audit
	@echo "CI-style checks completed"

# Quick start for new developers
quickstart: setup build example
	@echo "Quick start completed! Your development environment is ready."
	@echo ""
	@echo "Next steps:"
	@echo "  - Try: make test"
	@echo "  - Try: make check"
	@echo "  - Explore examples in the examples/ directory"
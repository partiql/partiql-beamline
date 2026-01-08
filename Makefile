# Beamline Makefile
# This Makefile provides convenient commands for building, testing, and managing the Beamline project.

.PHONY: help build build-release test test-all clean fmt check clippy doc install run examples bench coverage

# Default target
help: ## Show this help message
	@echo "Beamline available commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# Build commands
build: ## Build the project in debug mode
	cargo build

build-release: ## Build the project in release mode (optimized)
	cargo build --release

build-all: ## Build all features and targets
	cargo build --all-features --all-targets

# Test commands
test: ## Run tests
	cargo test

test-all: ## Run all tests including ignored ones
	cargo test -- --include-ignored

test-doc: ## Run documentation tests
	cargo test --doc

# Code quality commands
check: ## Check the project for errors without building
	cargo check

clippy: ## Run clippy linter (strict - treats warnings as errors)
	cargo clippy --all-targets --all-features -- -D warnings

clippy-dev: ## Run clippy linter for development (warnings only, non-blocking)
	cargo clippy --all-targets --all-features

fmt: ## Format the code
	cargo fmt

fmt-check: ## Check if code is formatted correctly
	cargo fmt -- --check

# Documentation
doc: ## Generate documentation
	cargo doc --no-deps --open

doc-all: ## Generate documentation for all dependencies
	cargo doc --open

# Installation and running
install: ## Install the CLI binary
	cargo install --path partiql-beamline-cli

run: ## Run the CLI (use ARGS="..." to pass arguments)
	cargo run --bin beamline -- $(ARGS)

# Example commands
examples: ## Run example data generation
	@echo "Running basic sensor data generation example..."
	cargo run --bin beamline -- gen data --seed 12345 --start-auto --sample-count 5 --script-path partiql-beamline-sim/tests/scripts/sensors.ion
	@echo ""
	@echo "Running shape inference example..."
	cargo run --bin beamline -- infer-shape --seed 12345 --start-auto --script-path partiql-beamline-sim/tests/scripts/sensors.ion

# Benchmarking
bench: ## Run benchmarks
	cargo bench

# Coverage (requires cargo-tarpaulin: cargo install cargo-tarpaulin)
coverage: ## Generate test coverage report
	cargo tarpaulin --out Html --output-dir coverage

# Maintenance commands
clean: ## Clean build artifacts
	cargo clean

clean-all: ## Clean all artifacts including target directory
	rm -rf target/
	rm -rf coverage/

# Development setup
setup: ## Set up development environment
	@echo "Installing required tools..."
	rustup component add rustfmt clippy
	@echo "Development environment setup complete!"

# Release preparation
pre-release: fmt clippy test doc ## Run all checks before release
	@echo "Pre-release checks completed successfully!"

# Quick development cycle
dev: fmt clippy-dev test ## Quick development cycle: format, lint, test

# Workspace commands
workspace-check: ## Check all workspace members
	cargo check --workspace

workspace-test: ## Test all workspace members
	cargo test --workspace

workspace-build: ## Build all workspace members
	cargo build --workspace

# Book building (if mdbook is installed)
book: ## Build the documentation book
	@if command -v mdbook >/dev/null 2>&1; then \
		cd partiql-beamline-book && mdbook build; \
	else \
		echo "mdbook not found. Install with: cargo install mdbook"; \
	fi

book-serve: ## Serve the documentation book locally
	@if command -v mdbook >/dev/null 2>&1; then \
		cd partiql-beamline-book && mdbook serve; \
	else \
		echo "mdbook not found. Install with: cargo install mdbook"; \
	fi

# Docker commands (if Dockerfile exists)
docker-build: ## Build Docker image
	@if [ -f Dockerfile ]; then \
		docker build -t partiql-beamline .; \
	else \
		echo "Dockerfile not found"; \
	fi

# Git hooks setup
hooks: ## Set up git hooks
	@echo "Setting up git hooks..."
	@mkdir -p .git/hooks
	@echo '#!/bin/sh\nmake fmt-check && make clippy && make test' > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks installed!"

# Show project info
info: ## Show project information
	@echo "Beamline Project Information:"
	@echo "======================================"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Project structure:"
	@find . -name "Cargo.toml" -not -path "./target/*" | head -10
	@echo ""
	@echo "Available binaries:"
	@cargo metadata --format-version 1 | grep -o '"name":"[^"]*"' | grep -v "partiql-beamline" | head -5

# All-in-one commands
all: build test clippy fmt-check doc ## Build, test, lint, and generate docs

ci: fmt-check clippy test-all ## Run CI checks (format, lint, test)
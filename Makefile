.PHONY: build release install clean test help

# Extract version from Cargo.toml
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Default target
help:
	@echo "Available commands:"
	@echo "  make build    - Build release binary"
	@echo "  make install  - Install to ~/.cargo/bin"
	@echo "  make release  - Create git tag and push (triggers GitHub Actions)"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make test     - Run tests"
	@echo "  make version  - Show current version"

# Show current version
version:
	@echo "Current version: $(VERSION)"

# Build release binary
build:
	cargo build --release

# Install locally
install: build
	cargo install --path .

# Run tests
test:
	cargo test
	cargo clippy

# Clean build artifacts
clean:
	cargo clean

# Create release tag and push
release:
	@echo "Creating release v$(VERSION)..."
	@if git rev-parse "v$(VERSION)" >/dev/null 2>&1; then \
		echo "Error: Tag v$(VERSION) already exists!"; \
		echo "Please update version in Cargo.toml first."; \
		exit 1; \
	fi
	@echo "Committing any pending changes..."
	git add -A
	git commit -m "Release v$(VERSION)" --allow-empty
	git push origin main
	@echo "Creating tag v$(VERSION)..."
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	@echo "Pushing tag to origin..."
	git push origin "v$(VERSION)"
	@echo ""
	@echo "✅ Release v$(VERSION) created and pushed!"
	@echo "GitHub Actions will now build and publish the release."
	@echo "Check: https://github.com/dongri/coins/actions"

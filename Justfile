# Install development dependencies
deps:
	rustup component add clippy
	rustup component add rustfmt
	rustup component add rls rust-analysis rust-src
	type cargo-readme >/dev/null || cargo +stable install cargo-readme

# Check for mistakes
lint:
	cargo clippy

# Reformat the code
fmt:
	cargo fmt

# Generate the docs
doc:
	cargo doc

# Open the docs in a browser
doc-open: doc
	cargo doc --open

# Update README.md
readme:
	cargo readme -o README.md

# Run the tests
test:
	cargo test

# Install the crate from this local repo
install:
	cargo install --path . --debug --force

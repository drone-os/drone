# Check for mistakes
lint:
	rustup component add clippy
	cargo clippy

# Reformat the code
fmt:
	rustup component add rustfmt
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
	cargo install --path . --force

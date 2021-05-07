# Install dependencies
deps:
	type cargo-readme >/dev/null || cargo +stable install cargo-readme

# Reformat the source code
fmt:
	cargo fmt

# Check the source code for mistakes
lint:
	cargo clippy

# Build the documentation
doc:
	cargo doc

# Open the documentation in a browser
doc-open: doc
	cargo doc --open

# Run the tests
test:
	cargo test

# Install the binaries
install:
	cargo install --path . --debug --force

# Update README.md
readme:
	cargo readme -o README.md

# Bump the version
version-bump version:
	sed -i "s/\(api\.drone-os\.com\/drone-core\/\)[0-9]\+\(\.[0-9]\+\)\+/\1$(echo {{version}} | sed 's/\(.*\)\.[0-9]\+/\1/')/" \
		config/Cargo.toml
	sed -i '/\[.*\]/h;/version = ".*"/{x;s/\[package\]/version = "{{version}}"/;t;x}' \
		Cargo.toml config/Cargo.toml
	sed -i '/\[.*\]/h;/version = "=.*"/{x;s/\[.*drone-.*\]/version = "={{version}}"/;t;x}' \
		Cargo.toml

# Publish to crates.io
publish:
	cd config && cargo publish
	sleep 30
	cargo publish

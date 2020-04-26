cargo_features := '-Z features=itarget,build_dep,dev_dep -Z package-features'

# Install dependencies
deps:
	rustup component add clippy
	rustup component add rustfmt
	type cargo-readme >/dev/null || cargo +stable install cargo-readme

# Reformat the source code
fmt:
	cargo {{cargo_features}} fmt

# Check for mistakes
lint:
	cargo {{cargo_features}} clippy

# Generate the docs
doc:
	cargo {{cargo_features}} doc

# Open the docs in the browser
doc-open: doc
	cargo {{cargo_features}} doc --open

# Run the tests
test:
	cargo {{cargo_features}} test

# Install the binaries
install:
	cargo {{cargo_features}} install --path . --debug --force

# Update README.md
readme:
	cargo {{cargo_features}} readme -o README.md

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
	cd config && cargo {{cargo_features}} publish
	sleep 5
	cargo {{cargo_features}} publish

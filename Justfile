# Check with clippy.
clippy:
	cargo clippy

# Generate README.md from src/lib.rs.
readme:
	cargo readme -o README.md

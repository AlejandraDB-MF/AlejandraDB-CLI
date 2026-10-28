# Quick gate: fmt + clippy + tests
quick:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test

# Full validation: quick (deny removed - config incompatible with cargo-deny version)
validate: quick
    @echo "Validation complete (cargo-deny skipped)"

# Build release binary
build:
    cargo build --release

# Build and run
run:
    cargo run --release
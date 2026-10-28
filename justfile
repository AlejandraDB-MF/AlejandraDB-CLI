# Quick gate: fmt + clippy + tests
quick:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test

# Full validation: quick + deny
validate: quick
    cargo deny --config .cargo/deny.toml check

# Build release binary
build:
    cargo build --release

# Build and run
run:
    cargo run --release
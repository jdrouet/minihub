# minihub development commands
# Install just: cargo install just
# Run `just --list` to see all available commands.

# Default recipe: run all checks
default: check

# Format all code
fmt:
    cargo fmt --all

# Check formatting (CI mode — fails on diff)
fmt-check:
    cargo fmt -- --check

# Run clippy with strict warnings
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --all

# Run tests with output shown
test-verbose:
    cargo test --all -- --nocapture

# Run coverage (terminal summary)
cov:
    cargo llvm-cov --workspace

# Run coverage and generate HTML report
cov-html:
    cargo llvm-cov --workspace --html
    @echo "Report: target/llvm-cov/html/index.html"

# Run coverage and generate LCOV file
cov-lcov:
    cargo llvm-cov --workspace --lcov --output-path lcov.info

# Run all checks (fmt + clippy + test) — use before committing
check: fmt-check clippy test

# Build the workspace
build:
    cargo build --all

# Build in release mode
build-release:
    cargo build --all --release

# Run the minihubd binary
run:
    cargo run --bin minihubd

# Clean build artifacts
clean:
    cargo clean

# Watch for changes and run tests (requires cargo-watch)
watch:
    cargo watch -x 'test --all'

# Check dependency tree for issues
deps:
    cargo tree --workspace --depth 1

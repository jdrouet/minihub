# minihub build recipes

# Format all code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run clippy with warnings as errors
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --all

# Run coverage and show summary
cov:
    cargo llvm-cov

# Generate HTML coverage report
cov-html:
    cargo llvm-cov --html
    @echo "Coverage report: target/llvm-cov/html/index.html"

# Run all quality checks (fmt + clippy + test)
check: fmt-check clippy test

# Build the Leptos dashboard WASM bundle
build-dashboard:
    cd crates/adapters/adapter_dashboard_leptos && trunk build --release

# Build minihubd binary
build-minihubd:
    cargo build --release --bin minihubd

# Build everything (dashboard + minihubd)
build-all: build-dashboard build-minihubd

# Clean build artifacts
clean:
    cargo clean
    rm -rf crates/adapters/adapter_dashboard_leptos/dist

# Run minihubd with the built dashboard
run: build-dashboard
    MINIHUB_DASHBOARD_DIR=crates/adapters/adapter_dashboard_leptos/dist cargo run --bin minihubd

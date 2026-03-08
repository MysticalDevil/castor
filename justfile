# Castor Project Tasks

# Default task: list all commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Run the TUI
tui:
    cargo run -- tui

# List sessions using real Gemini data
list:
    cargo run -- list

# Run the doctor command to check environment
doctor:
    cargo run -- doctor

# Run all tests
test:
    cargo test

# Run tests and generate coverage report
coverage:
    cargo tarpaulin --ignore-tests --output-dir . --out Lcov

# CI Gate: Verify that coverage meets the minimum threshold (40%)
check-coverage:
    cargo tarpaulin --ignore-tests --fail-under 40

# Run the rich data generator script (requires python3)
gen-test-data:
    python3 scripts/generate_test_data.py

# Generate 2000+ sessions for stress testing
gen-stress-data:
    python3 scripts/generate_test_data.py 2000

# List sessions using the generated test data
test-list: gen-test-data
    cargo run -- --config test_config.json list

# Run the TUI using the generated test data
test-tui: gen-test-data
    cargo run -- --config test_config.json tui

# Stress test: Run CLI list on 2000+ sessions
stress-cli: gen-stress-data
    time cargo run -- --config test_config.json list > /dev/null

# Stress test: Run TUI on 2000+ sessions
stress-tui: gen-stress-data
    cargo run -- --config test_config.json tui

# Repeatable perf benchmarks (ignored tests)
perf-bench:
    cargo test --test perf_bench -- --ignored --nocapture

# Main Quality Gate
check:
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
    just test
    just check-coverage

# Format the code
fmt:
    cargo fmt --all

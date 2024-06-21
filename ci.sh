#!/bin/bash

set -e

# Build
cargo build

# Check formatting
cargo fmt -- --check

# Run Clippy
cargo clippy -- -D warnings

# Run tests
cargo test

echo "CI checks passed successfully."
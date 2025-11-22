#!/bin/bash

# -----------------------------------------------------------------------------
# Bash Strict Mode
# -----------------------------------------------------------------------------
# -e: Exit immediately if a command exits with a non-zero status.
# -u: Treat unset variables as an error and exit immediately.
# -o pipefail: The return value of a pipeline is the status of
#              the last command to exit with a non-zero status.
set -euo pipefail

echo "🚀 Starting CI checks..."

# -----------------------------------------------------------------------------
# 1. Formatting Check (Fastest)
# -----------------------------------------------------------------------------
# We run this first to fail fast if code style guidelines are not met.
# --all: Checks all packages in the workspace.
echo "🎨 Checking formatting..."
cargo fmt --all -- --check

# -----------------------------------------------------------------------------
# 2. Linting & Static Analysis (Clippy)
# -----------------------------------------------------------------------------
# Runs clippy to catch common mistakes and improve code quality.
# --workspace: Checks all crates in the workspace.
# --all-targets: Checks lib, bins, tests, benchmarks, and examples.
# -D warnings: Fails the build if any warning is found.
echo "linter Checking lints..."
cargo clippy --workspace --all-targets -- -D warnings

# -----------------------------------------------------------------------------
# 3. Testing
# -----------------------------------------------------------------------------
# Runs

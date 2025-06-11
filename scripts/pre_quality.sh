#!/bin/bash

# Pre-Quality Gate Script for OpenRouter API
# This script runs comprehensive quality checks before any implementation

set -e  # Exit on any error

echo "🔍 Running Pre-Quality Checks..."
echo "================================="

# Check Rust toolchain
echo "📋 Checking Rust toolchain..."
rustc --version
cargo --version

# Format check
echo "🎨 Checking code formatting..."
if ! cargo fmt --check; then
    echo "❌ Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Lint check
echo "📝 Running clippy lints..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy warnings found. Fix before proceeding."
    exit 1
fi

# Build check
echo "🔨 Building project..."
if ! cargo check --all-targets --all-features; then
    echo "❌ Build failed. Fix compilation errors."
    exit 1
fi

# Test check
echo "🧪 Running tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed. Fix failing tests."
    exit 1
fi

# Security audit (if available)
echo "🔒 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    if ! cargo audit; then
        echo "⚠️  Security vulnerabilities found. Review and address."
        exit 1
    fi
else
    echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"
fi

# Documentation check
echo "📚 Checking documentation builds..."
if ! cargo doc --no-deps --all-features; then
    echo "❌ Documentation build failed."
    exit 1
fi

echo "✅ All pre-quality checks passed!"
echo "================================="
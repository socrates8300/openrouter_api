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

# Lint check with rustls (default)
echo "📝 Running clippy lints (rustls)..."
if ! cargo clippy --all-targets --features tls-rustls -- -D warnings; then
    echo "❌ Clippy warnings found. Fix before proceeding."
    exit 1
fi

# Lint check with native-tls
echo "📝 Running clippy lints (native-tls)..."
if ! cargo clippy --all-targets --no-default-features --features tls-native-tls -- -D warnings; then
    echo "❌ Clippy warnings found. Fix before proceeding."
    exit 1
fi

# Build check with rustls (default)
echo "🔨 Building project (rustls)..."
if ! cargo check --all-targets --features tls-rustls; then
    echo "❌ Build failed. Fix compilation errors."
    exit 1
fi

# Build check with native-tls
echo "🔨 Building project (native-tls)..."
if ! cargo check --all-targets --no-default-features --features tls-native-tls; then
    echo "❌ Build failed. Fix compilation errors."
    exit 1
fi

# Test check with rustls (default)
echo "🧪 Running tests (rustls)..."
if ! cargo test --features tls-rustls; then
    echo "❌ Tests failed. Fix failing tests."
    exit 1
fi

# Test check with native-tls
echo "🧪 Running tests (native-tls)..."
if ! cargo test --no-default-features --features tls-native-tls; then
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

# Documentation check with rustls (default)
echo "📚 Checking documentation builds (rustls)..."
if ! cargo doc --no-deps --features tls-rustls; then
    echo "❌ Documentation build failed."
    exit 1
fi

# Documentation check with native-tls
echo "📚 Checking documentation builds (native-tls)..."
if ! cargo doc --no-deps --no-default-features --features tls-native-tls; then
    echo "❌ Documentation build failed."
    exit 1
fi

echo "✅ All pre-quality checks passed!"
echo "================================="

#!/bin/bash

# Forge Core Setup Script
# Automatically configures the repository after cloning

set -e

echo "🔨 Setting up Forge Core..."

# Set correct default repository for gh CLI
if command -v gh &> /dev/null; then
    echo "📌 Setting namastexlabs/forge-core as default repository..."
    gh repo set-default namastexlabs/forge-core
    echo "✅ Default repository configured"
else
    echo "⚠️  GitHub CLI (gh) not found - skipping default repo setup"
    echo "   Install gh: https://cli.github.com/"
fi

# Install pnpm dependencies
if command -v pnpm &> /dev/null; then
    echo "📦 Installing dependencies..."
    pnpm install
    echo "✅ Dependencies installed"
else
    echo "⚠️  pnpm not found - skipping dependency installation"
    echo "   Install pnpm: https://pnpm.io/installation"
fi

# Check for Rust
if command -v cargo &> /dev/null; then
    echo "🦀 Rust detected: $(rustc --version)"
else
    echo "⚠️  Rust not found"
    echo "   Install Rust: https://rustup.rs/"
fi

# Check for required cargo tools
if command -v cargo &> /dev/null; then
    if ! command -v cargo-watch &> /dev/null; then
        echo "📦 Installing cargo-watch..."
        cargo install cargo-watch
    fi

    if ! command -v sqlx &> /dev/null; then
        echo "📦 Installing sqlx-cli..."
        cargo install sqlx-cli
    fi
fi

echo ""
echo "✨ Setup complete!"
echo ""
echo "📚 Next steps:"
echo "   1. Run 'pnpm run dev' to start the development server"
echo "   2. See README.md for more information"
echo ""

#!/bin/bash

set -e  # Exit on any error

echo "🧹 Cleaning previous builds..."
rm -rf npx-cli/dist
mkdir -p npx-cli/dist/macos-arm64

echo "🔨 Building frontend..."
(cd frontend && npm run build)

echo "🔨 Building Rust binaries..."
cargo build --release --manifest-path Cargo.toml
cargo build --release --bin mcp_task_server --manifest-path Cargo.toml

echo "📦 Creating distribution package..."

# Copy the main binary
cp target/release/server automagik-forge
zip -q automagik-forge.zip automagik-forge
rm -f automagik-forge 
mv automagik-forge.zip npx-cli/dist/macos-arm64/automagik-forge.zip

# Copy the MCP binary
cp target/release/mcp_task_server automagik-forge-mcp
zip -q automagik-forge-mcp.zip automagik-forge-mcp
rm -f automagik-forge-mcp
mv automagik-forge-mcp.zip npx-cli/dist/macos-arm64/automagik-forge-mcp.zip

echo "✅ NPM package ready!"
echo "📁 Files created:"
echo "   - npx-cli/dist/macos-arm64/automagik-forge.zip"
echo "   - npx-cli/dist/macos-arm64/automagik-forge-mcp.zip"

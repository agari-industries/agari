#!/bin/bash
set -e

# Build script for Agari web frontend
# This script builds the WASM module and the web frontend

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🦀 Building Agari WASM module..."
cd "$ROOT_DIR"

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build WASM
wasm-pack build crates/agari-wasm --target web --out-dir ../../web/src/lib/wasm

echo "📦 Building web frontend..."
cd "$ROOT_DIR/web"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📥 Installing npm dependencies..."
    npm install
fi

# Build
npm run build

echo ""
echo "✅ Build complete!"
echo ""
echo "📁 Output: web/dist/"
echo ""
echo "To preview locally:"
echo "  cd web && npm run preview"
echo ""
echo "To deploy, copy the contents of web/dist/ to your static host."

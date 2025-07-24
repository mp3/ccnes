#!/bin/bash

set -e

echo "Building CCNES..."

# Build native version
echo "Building native version..."
cargo build --release

# Build WASM version
echo "Building WASM version..."
cd wasm
wasm-pack build --target web --out-dir pkg
cd ..

echo "Build complete!"
echo ""
echo "To run the native version:"
echo "  ./target/release/ccnes <rom_file>"
echo ""
echo "To run the WASM version:"
echo "  1. cd wasm/www"
echo "  2. python3 -m http.server 8000"
echo "  3. Open http://localhost:8000 in your browser"
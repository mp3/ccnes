#\!/bin/bash

echo "Building CCNES NES Emulator..."

# Build core library
echo "Building core library..."
cd core
cargo build --release
cd ..

# Build WebAssembly module
echo "Building WebAssembly module..."
cd wasm
./build.sh
cd ..

# Build native frontend (if SDL2 is available)
if command -v sdl2-config &> /dev/null; then
    echo "Building native SDL2 frontend..."
    cd native
    cargo build --release
    cd ..
    echo "Native build complete: ./target/release/ccnes"
else
    echo "SDL2 not found, skipping native build"
    echo "To build native version, install SDL2:"
    echo "  macOS: brew install sdl2"
    echo "  Ubuntu: sudo apt-get install libsdl2-dev"
fi

echo ""
echo "Build complete\!"
echo ""
echo "To run:"
echo "  Native: ./target/release/ccnes <rom.nes>"
echo "  Web: cd wasm && ./serve.py"
EOF < /dev/null
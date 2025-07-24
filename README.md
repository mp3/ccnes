# CCNES - Cross-platform NES Emulator

A Nintendo Entertainment System (NES) emulator written in Rust with support for both native desktop and WebAssembly targets.

## Features

- 6502 CPU emulation
- PPU (Picture Processing Unit) for graphics
- APU (Audio Processing Unit) for sound
- Support for mappers 0-3
- Cross-platform: runs natively and in web browsers
- Keyboard input support

## Project Structure

- `core/` - Platform-independent emulation core
- `native/` - Native desktop application using SDL2
- `wasm/` - WebAssembly implementation for browsers

## Building

### Prerequisites

- Rust toolchain (1.70+)
- wasm-pack (for WebAssembly builds)
- SDL2 development libraries (for native builds)

### Build Commands

```bash
# Build everything
./build.sh

# Build only native version
cargo build --release

# Build only WASM version
cd wasm && wasm-pack build --target web --out-dir pkg
```

## Running

### Native Version

```bash
./target/release/ccnes path/to/rom.nes
```

Controls:
- Arrow Keys: D-Pad
- Z: A Button
- X: B Button
- Enter: Start
- Right Shift: Select
- R: Reset
- Escape: Quit

### WebAssembly Version

1. Build the WASM module
2. Serve the web files:
   ```bash
   cd wasm/www
   python3 -m http.server 8000
   ```
3. Open http://localhost:8000 in your browser
4. Click "Load ROM" to select a .nes file

## Current Status

This is a basic implementation with:
- Basic CPU with BRK and NOP instructions (more to be implemented)
- Memory bus and cartridge loading
- PPU and APU structure (rendering/audio not yet implemented)
- Basic mapper support (0-3)

## TODO

- Complete 6502 instruction set
- Implement PPU rendering
- Implement APU audio synthesis
- Add more mapper support
- Save states
- Debugger interface

## License

MIT
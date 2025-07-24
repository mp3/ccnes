# CCNES - Cross-platform NES Emulator

A Nintendo Entertainment System (NES) emulator written in Rust with support for both native desktop and WebAssembly targets.

## Features

### Core Emulation
- **CPU**: Complete 6502 processor emulation with all official opcodes
- **PPU**: Full Picture Processing Unit implementation
  - Background rendering with scrolling
  - Sprite rendering with proper priorities
  - Sprite 0 hit detection
  - Accurate NES color palette
- **APU**: Audio Processing Unit with all channels
  - 2 Pulse channels
  - Triangle channel
  - Noise channel
  - DMC channel (basic support)
- **Mappers**: Support for mappers 0-3 (covers many classic games)
- **Controllers**: Standard NES controller support

### Platforms
- **Native**: SDL2-based desktop application with audio
- **WebAssembly**: Browser-based version with Web Audio API

## Building

### Prerequisites
- Rust toolchain (1.70+)
- For native builds: SDL2 development libraries
- For WASM builds: wasm-pack

### Quick Build
```bash
./build.sh
```

This will build both the WebAssembly and native versions (if SDL2 is available).

### Manual Build

#### Core Library
```bash
cd core
cargo build --release
```

#### Native Frontend
```bash
# Install SDL2 first:
# macOS: brew install sdl2
# Linux: sudo apt-get install libsdl2-dev

cd native
cargo build --release
```

#### WebAssembly Frontend
```bash
cd wasm
wasm-pack build --target web --out-dir pkg
```

## Running

### Native Version
```bash
./target/release/ccnes [OPTIONS] <ROM_FILE>

Options:
  -s, --scale <SCALE>  Window scale factor [default: 3]
  -f, --fullscreen     Start in fullscreen mode
```

### Web Version
```bash
cd wasm
./serve.py
# Open http://localhost:8000 in your browser
```

## Controls

| Key | NES Button |
|-----|------------|
| Arrow Keys | D-Pad |
| Z or A | A Button |
| X or S | B Button |
| Enter | Start |
| Right Shift | Select |
| R | Reset (Native) |
| F11 | Toggle Fullscreen (Native) |
| Escape | Quit (Native) |

## Architecture

The project is organized into three main components:

### Core (`core/`)
Platform-independent NES emulation core:
- CPU emulation with cycle-accurate timing
- PPU with scanline-based rendering
- APU with real-time audio generation
- Memory-mapped I/O bus
- Cartridge and mapper support

### Native Frontend (`native/`)
SDL2-based desktop application:
- Hardware-accelerated rendering
- Real-time audio output
- Keyboard input handling
- Fullscreen support

### WebAssembly Frontend (`wasm/`)
Browser-based emulator:
- Canvas rendering
- Web Audio API integration
- File loading from local system
- Responsive controls

## Testing

The project includes comprehensive test suites:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test cpu_instructions
cargo test sprite_rendering
cargo test controller_input
cargo test apu_
```

## Performance

- Native version targets 60 FPS with frame limiting
- Audio output at 44.1kHz
- WebAssembly performance depends on browser and hardware

## Compatibility

Supports iNES format ROM files (.nes) with mappers 0-3, which includes many classic NES games.

## License

MIT

## Acknowledgments

- NES hardware documentation from NesDev Wiki
- 6502 instruction set references
- PPU and APU timing information from various emulation resources
EOF < /dev/null
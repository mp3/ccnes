# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

### Full Build
```bash
./build.sh  # Builds core, WASM, and native (if SDL2 available)
```

### Core Library
```bash
cd core
cargo build --release
cargo test
cargo bench  # Run performance benchmarks
```

### Native Frontend (SDL2 required)
```bash
# Install SDL2 first:
# macOS: brew install sdl2
# Linux: sudo apt-get install libsdl2-dev

cd native
cargo build --release
cargo run --release -- <ROM_FILE>
```

### WebAssembly Frontend
```bash
cd wasm
./build.sh  # Builds WASM module and creates serve.py
./serve.py  # Starts web server on http://localhost:8000
```

## Testing Commands

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test cpu_       # CPU tests
cargo test ppu_       # PPU tests
cargo test apu_       # Audio tests
cargo test mapper     # Mapper tests
cargo test savestate  # Save state tests

# Run a single test
cargo test test_lda_immediate

# Run benchmarks
cd core && cargo bench
```

## Linting and Type Checking

```bash
cargo clippy --all-targets --all-features
cargo fmt --check
```

## Architecture Overview

### Core Library (`core/`)
The platform-independent NES emulation engine consisting of:

- **CPU (`cpu/`)**: 6502 processor emulation
  - `mod.rs`: Main CPU struct with step execution
  - `instructions.rs`: Instruction implementations (ADC, LDA, etc.)
  - `addressing.rs`: Addressing mode calculations
  - `opcodes.rs`: Opcode lookup table
  - `optimized.rs`: Performance-critical optimizations

- **PPU (`ppu/`)**: Picture Processing Unit
  - `mod.rs`: Main PPU with scanline-based rendering
  - `palette.rs`: NES color palette data
  - `optimized.rs`: Rendering optimizations

- **APU (`apu/`)**: Audio Processing Unit
  - `mod.rs`: All 5 audio channels (2 pulse, triangle, noise, DMC)
  - `filters.rs`: Audio filtering (low-pass, high-pass)
  - `resampler.rs`: Sample rate conversion
  - `buffer.rs`: Ring buffer for audio output

- **Bus (`bus.rs`)**: Memory-mapped I/O system connecting components
  - Handles CPU memory map ($0000-$FFFF)
  - Routes reads/writes to appropriate components
  - Manages OAM DMA transfers

- **Cartridge (`cartridge/`)**: ROM loading and mapper support
  - `mod.rs`: iNES format parsing and mapper selection
  - `mappers/`: Individual mapper implementations (0-5, 7, 9, 11, 66)
  - Each mapper handles PRG/CHR banking and mirroring

- **NES (`nes.rs`)**: Main emulator class
  - Coordinates CPU, PPU, APU timing
  - Provides step() and run_frame() methods

### Platform Frontends

- **Native (`native/`)**: SDL2-based desktop application
  - Handles video rendering, audio output, and input
  - Supports fullscreen and window scaling

- **WASM (`wasm/`)**: Browser-based emulator
  - Canvas rendering and Web Audio API
  - File loading interface

## Key Implementation Details

### Timing
- CPU and PPU run in lockstep
- 1 CPU cycle = 3 PPU cycles
- PPU generates NMI at vblank (scanline 241)
- Frame timing: 262 scanlines × 341 PPU cycles

### Memory Map
- $0000-$1FFF: RAM (mirrored)
- $2000-$3FFF: PPU registers (mirrored)
- $4000-$4017: APU/IO registers
- $4020-$FFFF: Cartridge space

### Audio Processing
- APU generates samples at NES frequency (~1.79 MHz)
- Resampled to 44.1kHz for output
- Filtering applied to match NES audio characteristics

### Save States
- Full emulator state serialization using bincode
- Includes CPU, PPU, APU, and mapper states
- Multiple save slots supported

## Performance Notes

- Optimized CPU instruction dispatch for common opcodes
- PPU uses precomputed tables for attribute lookups
- Inline functions for critical paths
- Benchmarks available: `cd core && cargo bench`
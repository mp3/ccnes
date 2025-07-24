# CCNES Native Frontend

This is the native SDL2 frontend for the CCNES NES emulator, providing a desktop application with full audio and video output.

## Features

- SDL2-based rendering with configurable scaling
- Real-time audio output via SDL2 audio
- Keyboard input support
- Command-line interface
- Frame rate limiting (60 FPS)
- Hardware-accelerated rendering

## Building

Prerequisites:
- Rust toolchain
- SDL2 development libraries

### macOS
```bash
brew install sdl2
cargo build --release
```

### Linux (Ubuntu/Debian)
```bash
sudo apt-get install libsdl2-dev
cargo build --release
```

### Windows
Follow the SDL2 setup instructions for Windows, then:
```bash
cargo build --release
```

## Usage

```bash
ccnes [OPTIONS] <ROM_PATH>

Arguments:
  <ROM_PATH>  ROM file to load

Options:
  -s, --scale <SCALE>  Scale factor for display [default: 3]
  -f, --fullscreen     Start in fullscreen mode
  -h, --help           Print help
  -V, --version        Print version
```

Example:
```bash
./ccnes -s 4 game.nes
```

## Controls

### Keyboard Mapping

| Key | NES Button |
|-----|------------|
| Arrow Keys | D-Pad |
| Z or A | A Button |
| X or S | B Button |
| Enter | Start |
| Right Shift | Select |
| R | Reset |
| F11 | Toggle Fullscreen |
| Escape | Quit |

## Performance

The emulator targets 60 FPS with frame limiting. Audio is output at 44.1kHz mono.

## Troubleshooting

### No Audio
- Ensure SDL2 audio drivers are properly installed
- Check system audio settings

### Poor Performance
- Try reducing the scale factor
- Ensure you're running the release build (`--release`)

### Controller Not Responding
- Check that the window has focus
- Some keyboards may have limitations on simultaneous key presses
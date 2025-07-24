# CCNES WebAssembly Module

This is the WebAssembly build of the CCNES NES emulator, allowing it to run in web browsers.

## Features

- Full NES emulation running in the browser
- Canvas-based rendering with pixel-perfect scaling
- Web Audio API for sound output
- Keyboard controls
- ROM loading from local files

## Building

Prerequisites:
- Rust with `wasm32-unknown-unknown` target
- wasm-pack (`cargo install wasm-pack`)

To build:
```bash
./build.sh
```

## Running

After building:
```bash
./serve.py
```

Then open http://localhost:8000 in your browser.

## Controls

- **Arrow Keys**: D-Pad
- **Z or A**: A Button  
- **X or S**: B Button
- **Enter**: Start
- **Right Shift**: Select

## Browser Requirements

- Modern browser with WebAssembly support
- Web Audio API support
- Canvas 2D rendering

## Performance

The emulator aims for 60 FPS. Performance may vary based on:
- Browser and JavaScript engine
- Hardware capabilities
- ROM complexity

## Troubleshooting

- **No sound**: Click anywhere on the page to enable audio (browser requirement)
- **Poor performance**: Try using Chrome or Firefox for best performance
- **ROM won't load**: Ensure the file is a valid .nes ROM file
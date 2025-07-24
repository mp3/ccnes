import init, { WasmNes } from '../pkg/ccnes_wasm.js';

let nes = null;
let animationId = null;
let isPaused = false;
let controllerState = 0;

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const loadRomButton = document.getElementById('loadRom');
const romFileInput = document.getElementById('romFile');
const resetButton = document.getElementById('reset');
const pauseButton = document.getElementById('pause');

// Keyboard mapping
const keyMap = {
    'KeyZ': 0x80,     // A
    'KeyX': 0x40,     // B
    'ShiftRight': 0x20, // Select
    'Enter': 0x10,    // Start
    'ArrowUp': 0x08,  // Up
    'ArrowDown': 0x04, // Down
    'ArrowLeft': 0x02, // Left
    'ArrowRight': 0x01 // Right
};

async function main() {
    await init();
    console.log('WASM module initialized');
}

function gameLoop() {
    if (!isPaused && nes) {
        nes.run_frame();
        nes.render(ctx);
    }
    animationId = requestAnimationFrame(gameLoop);
}

function startEmulation() {
    if (!animationId) {
        gameLoop();
    }
    resetButton.disabled = false;
    pauseButton.disabled = false;
}

function stopEmulation() {
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }
}

loadRomButton.addEventListener('click', () => {
    romFileInput.click();
});

romFileInput.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    
    try {
        const arrayBuffer = await file.arrayBuffer();
        const romData = new Uint8Array(arrayBuffer);
        
        if (!nes) {
            nes = new WasmNes();
        }
        
        nes.load_rom(romData);
        console.log('ROM loaded:', file.name);
        
        isPaused = false;
        pauseButton.textContent = 'Pause';
        startEmulation();
    } catch (error) {
        console.error('Failed to load ROM:', error);
        alert('Failed to load ROM: ' + error.message);
    }
});

resetButton.addEventListener('click', () => {
    if (nes) {
        nes.reset();
        console.log('NES reset');
    }
});

pauseButton.addEventListener('click', () => {
    isPaused = !isPaused;
    pauseButton.textContent = isPaused ? 'Resume' : 'Pause';
});

// Keyboard input handling
document.addEventListener('keydown', (e) => {
    if (!nes || e.repeat) return;
    
    const bit = keyMap[e.code];
    if (bit) {
        e.preventDefault();
        controllerState |= bit;
        nes.set_controller(0, controllerState);
    }
});

document.addEventListener('keyup', (e) => {
    if (!nes) return;
    
    const bit = keyMap[e.code];
    if (bit) {
        e.preventDefault();
        controllerState &= ~bit;
        nes.set_controller(0, controllerState);
    }
});

// Prevent arrow keys from scrolling
window.addEventListener('keydown', (e) => {
    if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.code)) {
        e.preventDefault();
    }
});

// Initialize the app
main().catch(console.error);
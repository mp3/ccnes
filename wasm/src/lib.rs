use ccnes_core::{Cartridge, Nes};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct WasmNes {
    nes: Nes,
    framebuffer: Vec<u8>,
}

#[wasm_bindgen]
impl WasmNes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());
        
        console_log!("Initializing WASM NES emulator");
        
        Self {
            nes: Nes::new(),
            framebuffer: vec![0; 256 * 240 * 4], // RGBA format
        }
    }
    
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<(), JsValue> {
        console_log!("Loading ROM, size: {} bytes", rom_data.len());
        
        let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
            .map_err(|e| JsValue::from_str(&format!("Failed to load ROM: {}", e)))?;
        
        self.nes.load_cartridge(cartridge);
        console_log!("ROM loaded successfully");
        Ok(())
    }
    
    pub fn reset(&mut self) {
        console_log!("Resetting NES");
        self.nes.reset();
    }
    
    pub fn run_frame(&mut self) {
        self.nes.run_frame();
        
        // Get framebuffer from PPU and convert to RGBA
        let nes_framebuffer = self.nes.get_framebuffer();
        for y in 0..240 {
            for x in 0..256 {
                let pixel_index = y * 256 + x;
                let color = nes_framebuffer[pixel_index];
                
                // Extract RGB components from 0x00RRGGBB format
                let r = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = (color & 0xFF) as u8;
                
                let idx = pixel_index * 4;
                self.framebuffer[idx] = r;
                self.framebuffer[idx + 1] = g;
                self.framebuffer[idx + 2] = b;
                self.framebuffer[idx + 3] = 255;  // Alpha
            }
        }
    }
    
    pub fn render(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.framebuffer),
            256,
            240,
        )?;
        
        ctx.put_image_data(&image_data, 0.0, 0.0)?;
        Ok(())
    }
    
    pub fn set_controller(&mut self, controller: u8, state: u8) {
        if controller == 0 {
            self.nes.set_controller1(state);
        } else {
            self.nes.set_controller2(state);
        }
    }
    
    pub fn key_down(&mut self, key_code: &str) -> u8 {
        self.update_controller(key_code, true)
    }
    
    pub fn key_up(&mut self, key_code: &str) -> u8 {
        self.update_controller(key_code, false)
    }
    
    fn update_controller(&mut self, key_code: &str, pressed: bool) -> u8 {
        let mut state = 0u8;
        
        // Map keyboard to NES controller
        let bit = match key_code {
            "KeyZ" | "KeyA" => 0x80,  // A
            "KeyX" | "KeyS" => 0x40,  // B
            "ShiftRight" => 0x20,      // Select
            "Enter" => 0x10,           // Start
            "ArrowUp" => 0x08,         // Up
            "ArrowDown" => 0x04,       // Down
            "ArrowLeft" => 0x02,       // Left
            "ArrowRight" => 0x01,      // Right
            _ => return state,
        };
        
        if pressed {
            state |= bit;
        }
        
        state
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("CCNES WASM module loaded");
}
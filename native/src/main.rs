use ccnes_core::{Cartridge, Nes};
use clap::Parser;
use log::info;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::fs::File;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ROM file to load
    rom_path: String,
    
    /// Scale factor for display
    #[arg(short, long, default_value_t = 3)]
    scale: u32,
}

const NES_WIDTH: u32 = 256;
const NES_HEIGHT: u32 = 240;
const TARGET_FPS: u32 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();
    
    info!("Loading ROM: {}", args.rom_path);
    let rom_file = File::open(&args.rom_path)?;
    let cartridge = Cartridge::from_ines(rom_file)?;
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let window = video_subsystem
        .window("CCNES", NES_WIDTH * args.scale, NES_HEIGHT * args.scale)
        .position_centered()
        .build()?;
    
    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();
    
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        NES_WIDTH,
        NES_HEIGHT,
    )?;
    
    let mut event_pump = sdl_context.event_pump()?;
    let mut framebuffer = vec![0u8; (NES_WIDTH * NES_HEIGHT * 3) as usize];
    
    let mut frame_start = Instant::now();
    
    'running: loop {
        let mut controller_state = 0u8;
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::R => nes.reset(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Update controller state based on keyboard
        let keyboard_state = event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Z) {
            controller_state |= 0x80; // A button
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::X) {
            controller_state |= 0x40; // B button
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RShift) {
            controller_state |= 0x20; // Select
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Return) {
            controller_state |= 0x10; // Start
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up) {
            controller_state |= 0x08; // Up
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down) {
            controller_state |= 0x04; // Down
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
            controller_state |= 0x02; // Left
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
            controller_state |= 0x01; // Right
        }
        
        nes.set_controller1(controller_state);
        
        // Run one frame
        nes.run_frame();
        
        // TODO: Get actual framebuffer from PPU
        // For now, just fill with a test pattern
        for i in 0..framebuffer.len() {
            framebuffer[i] = (i % 255) as u8;
        }
        
        // Update texture and render
        texture.update(None, &framebuffer, (NES_WIDTH * 3) as usize)?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
        
        // Frame rate limiting
        let frame_elapsed = frame_start.elapsed();
        if frame_elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - frame_elapsed);
        }
        frame_start = Instant::now();
    }
    
    Ok(())
}
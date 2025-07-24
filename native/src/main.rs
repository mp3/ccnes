use ccnes_core::{Cartridge, Nes, Controller, ControllerButton, SaveStateError};
use clap::Parser;
use log::info;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod debugger_ui;
use debugger_ui::DebuggerUI;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ROM file to load
    rom_path: String,
    
    /// Scale factor for display
    #[arg(short, long, default_value_t = 3)]
    scale: u32,
    
    /// Start in fullscreen mode
    #[arg(short, long)]
    fullscreen: bool,
}

const NES_WIDTH: u32 = 256;
const NES_HEIGHT: u32 = 240;
const TARGET_FPS: u32 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

struct AudioOutput {
    samples: Arc<Mutex<Vec<f32>>>,
}

impl AudioCallback for AudioOutput {
    type Channel = f32;
    
    fn callback(&mut self, out: &mut [f32]) {
        let mut samples = self.samples.lock().unwrap();
        let available = samples.len().min(out.len());
        
        for i in 0..available {
            out[i] = samples[i];
        }
        
        // Fill rest with silence if needed
        for i in available..out.len() {
            out[i] = 0.0;
        }
        
        // Remove consumed samples
        samples.drain(0..available);
    }
}

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
    let audio_subsystem = sdl_context.audio()?;
    
    let mut window_builder = video_subsystem
        .window("CCNES", NES_WIDTH * args.scale, NES_HEIGHT * args.scale)
        .position_centered();
    
    if args.fullscreen {
        window_builder.fullscreen_desktop();
    }
    
    let window = window_builder.build()?;
    
    let mut canvas = window.into_canvas().accelerated().build()?;
    let texture_creator = canvas.texture_creator();
    
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        NES_WIDTH,
        NES_HEIGHT,
    )?;
    
    // Set up audio
    let audio_samples = Arc::new(Mutex::new(Vec::new()));
    let audio_spec_desired = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(2048),
    };
    
    let audio_device = audio_subsystem.open_playback(
        None,
        &audio_spec_desired,
        |spec| {
            info!("Audio initialized: {} Hz, {} channels", spec.freq, spec.channels);
            AudioOutput {
                samples: audio_samples.clone(),
            }
        },
    )?;
    
    audio_device.resume();
    
    let mut event_pump = sdl_context.event_pump()?;
    let mut framebuffer = vec![0u8; (NES_WIDTH * NES_HEIGHT * 3) as usize];
    let mut controller = Controller::new();
    
    // Save state slots (10 slots)
    let mut save_states: Vec<Option<Vec<u8>>> = vec![None; 10];
    let mut current_save_slot = 0;
    
    // Debugger
    let mut debugger_ui = DebuggerUI::new();
    
    let mut frame_start = Instant::now();
    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::R => nes.reset(),
                        Keycode::F11 => {
                            use sdl2::video::FullscreenType;
                            let window = canvas.window_mut();
                            let fullscreen_type = window.fullscreen_state();
                            window.set_fullscreen(
                                if fullscreen_type == FullscreenType::Off {
                                    FullscreenType::Desktop
                                } else {
                                    FullscreenType::Off
                                }
                            ).ok();
                        }
                        // Save state
                        Keycode::F5 => {
                            match nes.save_state_to_vec() {
                                Ok(data) => {
                                    save_states[current_save_slot] = Some(data);
                                    info!("Saved state to slot {}", current_save_slot);
                                }
                                Err(e) => {
                                    info!("Failed to save state: {}", e);
                                }
                            }
                        }
                        // Load state
                        Keycode::F9 => {
                            if let Some(data) = &save_states[current_save_slot] {
                                match nes.load_state_from_slice(data) {
                                    Ok(()) => {
                                        info!("Loaded state from slot {}", current_save_slot);
                                    }
                                    Err(e) => {
                                        info!("Failed to load state: {}", e);
                                    }
                                }
                            } else {
                                info!("No save state in slot {}", current_save_slot);
                            }
                        }
                        // Select save slot 0-9
                        Keycode::Num0 => current_save_slot = 0,
                        Keycode::Num1 => current_save_slot = 1,
                        Keycode::Num2 => current_save_slot = 2,
                        Keycode::Num3 => current_save_slot = 3,
                        Keycode::Num4 => current_save_slot = 4,
                        Keycode::Num5 => current_save_slot = 5,
                        Keycode::Num6 => current_save_slot = 6,
                        Keycode::Num7 => current_save_slot = 7,
                        Keycode::Num8 => current_save_slot = 8,
                        Keycode::Num9 => current_save_slot = 9,
                        // Debugger toggle
                        Keycode::F10 => {
                            debugger_ui.toggle();
                        }
                        _ => {
                            // Pass other keys to debugger if active
                            debugger_ui.handle_key(keycode, &mut nes);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Update controller state based on keyboard
        let keyboard_state = event_pump.keyboard_state();
        controller.set_button(ControllerButton::A, 
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Z) ||
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A));
        controller.set_button(ControllerButton::B,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::X) ||
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S));
        controller.set_button(ControllerButton::SELECT,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RShift));
        controller.set_button(ControllerButton::START,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Return));
        controller.set_button(ControllerButton::UP,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Up));
        controller.set_button(ControllerButton::DOWN,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Down));
        controller.set_button(ControllerButton::LEFT,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left));
        controller.set_button(ControllerButton::RIGHT,
            keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right));
        
        nes.set_controller1_from_controller(&controller);
        
        // Run one frame (or step if debugger is active)
        if debugger_ui.is_active() && debugger_ui.get_debugger().is_paused() {
            // In debug mode, step through instructions
            for _ in 0..29780 { // Approximate cycles per frame
                nes.step();
                debugger_ui.update(&mut nes);
                if debugger_ui.get_debugger().is_paused() {
                    break;
                }
            }
        } else {
            nes.run_frame();
        }
        
        debugger_ui.update_frame();
        
        // Get audio samples and push to audio buffer
        let samples = nes.bus.apu.get_samples();
        if !samples.is_empty() {
            let mut audio_buffer = audio_samples.lock().unwrap();
            audio_buffer.extend_from_slice(&samples);
            
            // Prevent buffer overflow - keep only last ~0.5 seconds
            if audio_buffer.len() > 22050 {
                let start = audio_buffer.len() - 22050;
                audio_buffer.drain(0..start);
            }
        }
        
        // Get framebuffer from PPU and convert to RGB24
        let nes_framebuffer = nes.get_framebuffer();
        for y in 0..NES_HEIGHT as usize {
            for x in 0..NES_WIDTH as usize {
                let pixel_index = y * NES_WIDTH as usize + x;
                let color = nes_framebuffer[pixel_index];
                
                // Extract RGB components from 0x00RRGGBB format
                let r = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = (color & 0xFF) as u8;
                
                let offset = pixel_index * 3;
                framebuffer[offset] = r;
                framebuffer[offset + 1] = g;
                framebuffer[offset + 2] = b;
            }
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
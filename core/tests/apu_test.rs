use ccnes_core::{Cartridge, Nes};
use ccnes_core::cpu::CpuBus;

#[test]
fn test_apu_basic_functionality() {
    // Create a simple test ROM
    let rom_data = ccnes_core::test_rom::create_test_rom();
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load ROM");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Write to APU registers to set up a pulse channel
    nes.bus.write(0x4000, 0x8F);  // Pulse 1: duty cycle 2, max volume
    nes.bus.write(0x4002, 0xFF);  // Pulse 1: timer low byte
    nes.bus.write(0x4003, 0x00);  // Pulse 1: timer high byte, length counter
    nes.bus.write(0x4015, 0x01);  // Enable pulse 1
    
    // Run for a few cycles to let APU process
    for _ in 0..1000 {
        nes.step();
    }
    
    // Check APU status
    let status = nes.bus.read(0x4015);
    assert_eq!(status & 0x01, 0x01, "Pulse 1 should be enabled");
    
    // Get some audio samples
    let samples = nes.bus.apu.get_samples();
    assert!(!samples.is_empty(), "APU should generate audio samples");
}

#[test]
fn test_apu_all_channels() {
    let rom_data = ccnes_core::test_rom::create_test_rom();
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load ROM");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Enable all channels
    nes.bus.write(0x4015, 0x1F);
    
    // Set up pulse 1
    nes.bus.write(0x4000, 0x8F);
    nes.bus.write(0x4002, 0xFF);
    nes.bus.write(0x4003, 0x00);
    
    // Set up pulse 2
    nes.bus.write(0x4004, 0x8F);
    nes.bus.write(0x4006, 0x7F);
    nes.bus.write(0x4007, 0x00);
    
    // Set up triangle
    nes.bus.write(0x4008, 0xFF);
    nes.bus.write(0x400A, 0x3F);
    nes.bus.write(0x400B, 0x00);
    
    // Set up noise
    nes.bus.write(0x400C, 0x8F);
    nes.bus.write(0x400E, 0x04);
    nes.bus.write(0x400F, 0x00);
    
    // Run for a frame
    nes.run_frame();
    
    // Check that all channels are still enabled
    let status = nes.bus.read(0x4015);
    assert_eq!(status & 0x0F, 0x0F, "All sound channels should be enabled");
    
    // Verify samples were generated
    let samples = nes.bus.apu.get_samples();
    assert!(samples.len() > 100, "Should have generated many samples");
    
    // Verify samples aren't all zero
    let non_zero_samples = samples.iter().filter(|&&s| s != 0.0).count();
    assert!(non_zero_samples > 0, "Should have non-zero samples");
}
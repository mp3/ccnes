use ccnes_core::{Cartridge, Nes};
use std::io::Cursor;

// Simple audio test ROM that plays different notes
fn create_audio_test_rom() -> Vec<u8> {
    let mut rom = vec![0; 0x4000]; // 16KB PRG ROM
    
    // PRG ROM starts at 0x0000
    let mut offset = 0;
    
    // Initialize stack pointer
    rom[offset] = 0xA2; offset += 1; // LDX #$FF
    rom[offset] = 0xFF; offset += 1;
    rom[offset] = 0x9A; offset += 1; // TXS
    
    // Initialize APU
    rom[offset] = 0xA9; offset += 1; // LDA #$0F
    rom[offset] = 0x0F; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4015 (enable all channels)
    rom[offset] = 0x15; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    // Play a C note on pulse 1
    rom[offset] = 0xA9; offset += 1; // LDA #$88 (duty cycle 2, no sweep/envelope)
    rom[offset] = 0x88; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4000
    rom[offset] = 0x00; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$00 (no sweep)
    rom[offset] = 0x00; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4001
    rom[offset] = 0x01; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$42 (timer low for C4)
    rom[offset] = 0x42; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4002
    rom[offset] = 0x02; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$01 (timer high + length counter)
    rom[offset] = 0x01; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4003
    rom[offset] = 0x03; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    // Wait a bit
    rom[offset] = 0xA2; offset += 1; // LDX #$FF
    rom[offset] = 0xFF; offset += 1;
    rom[offset] = 0xA0; offset += 1; // LDY #$FF
    rom[offset] = 0xFF; offset += 1;
    
    // Wait loop
    rom[offset] = 0x88; offset += 1; // DEY
    rom[offset] = 0xD0; offset += 1; // BNE -1
    rom[offset] = 0xFD; offset += 1;
    rom[offset] = 0xCA; offset += 1; // DEX
    rom[offset] = 0xD0; offset += 1; // BNE -4
    rom[offset] = 0xF8; offset += 1;
    
    // Play an E note on pulse 2
    rom[offset] = 0xA9; offset += 1; // LDA #$88
    rom[offset] = 0x88; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4004
    rom[offset] = 0x04; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$00
    rom[offset] = 0x00; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4005
    rom[offset] = 0x05; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$50 (timer low for E4)
    rom[offset] = 0x50; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4006
    rom[offset] = 0x06; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$01
    rom[offset] = 0x01; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4007
    rom[offset] = 0x07; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    // Another wait
    rom[offset] = 0xA2; offset += 1; // LDX #$FF
    rom[offset] = 0xFF; offset += 1;
    rom[offset] = 0xA0; offset += 1; // LDY #$FF
    rom[offset] = 0xFF; offset += 1;
    
    rom[offset] = 0x88; offset += 1; // DEY
    rom[offset] = 0xD0; offset += 1; // BNE -1
    rom[offset] = 0xFD; offset += 1;
    rom[offset] = 0xCA; offset += 1; // DEX
    rom[offset] = 0xD0; offset += 1; // BNE -4
    rom[offset] = 0xF8; offset += 1;
    
    // Play a G note on triangle
    rom[offset] = 0xA9; offset += 1; // LDA #$81 (enable triangle)
    rom[offset] = 0x81; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $4008
    rom[offset] = 0x08; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$3B (timer low for G4)
    rom[offset] = 0x3B; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $400A
    rom[offset] = 0x0A; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    rom[offset] = 0xA9; offset += 1; // LDA #$00 (timer high)
    rom[offset] = 0x00; offset += 1;
    rom[offset] = 0x8D; offset += 1; // STA $400B
    rom[offset] = 0x0B; offset += 1;
    rom[offset] = 0x40; offset += 1;
    
    // Infinite loop
    rom[offset] = 0x4C; offset += 1; // JMP to self
    rom[offset] = (0x8000 + offset - 3) as u8; offset += 1;
    rom[offset] = ((0x8000 + offset - 4) >> 8) as u8; offset += 1;
    
    // Set reset vector to start of code (0x8000)
    rom[0x3FFC] = 0x00;
    rom[0x3FFD] = 0x80;
    
    // Create iNES header + PRG ROM
    let mut ines_rom = vec![];
    
    // iNES header
    ines_rom.extend_from_slice(b"NES\x1a"); // Magic
    ines_rom.push(1); // 1 * 16KB PRG ROM
    ines_rom.push(0); // 0 * 8KB CHR ROM
    ines_rom.push(0); // Mapper 0, vertical mirroring
    ines_rom.push(0); // Mapper 0
    ines_rom.extend_from_slice(&[0; 8]); // Padding
    
    // PRG ROM (16KB)
    ines_rom.extend_from_slice(&rom);
    
    ines_rom
}

#[test]
fn test_audio_generation() {
    let rom_data = create_audio_test_rom();
    let cartridge = Cartridge::from_ines(Cursor::new(rom_data)).unwrap();
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Run for a few frames to generate audio
    for _ in 0..60 {
        nes.run_frame();
        
        // Get audio samples
        let samples = nes.bus.apu.get_samples();
        
        // Verify we're generating audio samples
        assert!(!samples.is_empty(), "APU should generate audio samples");
        
        // Check that samples are in valid range
        for sample in &samples {
            assert!(*sample >= -1.0 && *sample <= 1.0, "Audio samples should be normalized");
        }
    }
}

#[test]
fn test_apu_direct() {
    use ccnes_core::Nes;
    
    let mut nes = Nes::new();
    
    // Enable all APU channels directly
    nes.bus.apu.write_register(0x4015, 0x0F);
    
    // Configure pulse 1 channel
    nes.bus.apu.write_register(0x4000, 0x8F); // Duty 2, max volume
    nes.bus.apu.write_register(0x4001, 0x00); // No sweep
    nes.bus.apu.write_register(0x4002, 0x42); // Timer low
    nes.bus.apu.write_register(0x4003, 0x08); // Timer high + trigger
    
    // Run APU for a bit
    for _ in 0..1000 {
        nes.bus.apu.step();
    }
    
    // Get samples - should have generated some
    let samples = nes.bus.apu.get_samples();
    assert!(!samples.is_empty(), "APU should generate samples");
    
    // Verify samples are in valid range
    for sample in &samples {
        assert!(*sample >= -1.0 && *sample <= 1.0, "Samples should be normalized");
    }
}
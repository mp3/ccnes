use std::fs::File;
use std::io::Write;

// Create an audio demo ROM that plays a simple melody
pub fn create_audio_demo_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    // iNES header
    rom.extend_from_slice(b"NES\x1A");  // Magic
    rom.push(1);  // 1x 16KB PRG ROM
    rom.push(0);  // 0x 8KB CHR ROM (not needed for audio)
    rom.push(0);  // Mapper 0, horizontal mirroring
    rom.push(0);  // Mapper 0 high nibble
    rom.extend_from_slice(&[0; 8]);  // Padding
    
    // PRG ROM (16KB)
    let mut prg = vec![0; 16384];
    
    // Build the program
    let mut offset = 0;
    
    // Initialize stack pointer
    prg[offset] = 0xA2; offset += 1; // LDX #$FF
    prg[offset] = 0xFF; offset += 1;
    prg[offset] = 0x9A; offset += 1; // TXS
    
    // Enable all APU channels
    prg[offset] = 0xA9; offset += 1; // LDA #$0F
    prg[offset] = 0x0F; offset += 1;
    prg[offset] = 0x8D; offset += 1; // STA $4015
    prg[offset] = 0x15; offset += 1;
    prg[offset] = 0x40; offset += 1;
    
    // Main melody loop
    let melody_start = offset;
    
    // Note table index
    prg[offset] = 0xA2; offset += 1; // LDX #$00
    prg[offset] = 0x00; offset += 1;
    
    // Play note loop
    let play_loop = offset;
    
    // Load note data from table at $C200
    prg[offset] = 0xBD; offset += 1; // LDA $C200,X
    prg[offset] = 0x00; offset += 1;
    prg[offset] = 0xC2; offset += 1;
    prg[offset] = 0xF0; offset += 1; // BEQ +50 (restart if zero)
    prg[offset] = 50; offset += 1;
    
    // Save note value
    prg[offset] = 0x48; offset += 1; // PHA
    
    // Set pulse 1 parameters
    // Duty cycle and volume
    prg[offset] = 0xA9; offset += 1; // LDA #$8F (duty 2, max volume)
    prg[offset] = 0x8F; offset += 1;
    prg[offset] = 0x8D; offset += 1; // STA $4000
    prg[offset] = 0x00; offset += 1;
    prg[offset] = 0x40; offset += 1;
    
    // No sweep
    prg[offset] = 0xA9; offset += 1; // LDA #$00
    prg[offset] = 0x00; offset += 1;
    prg[offset] = 0x8D; offset += 1; // STA $4001
    prg[offset] = 0x01; offset += 1;
    prg[offset] = 0x40; offset += 1;
    
    // Timer low (note frequency)
    prg[offset] = 0x68; offset += 1; // PLA
    prg[offset] = 0x8D; offset += 1; // STA $4002
    prg[offset] = 0x02; offset += 1;
    prg[offset] = 0x40; offset += 1;
    
    // Timer high and trigger
    prg[offset] = 0xA9; offset += 1; // LDA #$08 (length counter loaded)
    prg[offset] = 0x08; offset += 1;
    prg[offset] = 0x8D; offset += 1; // STA $4003
    prg[offset] = 0x03; offset += 1;
    prg[offset] = 0x40; offset += 1;
    
    // Wait for note duration
    prg[offset] = 0xA0; offset += 1; // LDY #$80
    prg[offset] = 0x80; offset += 1;
    let wait_outer = offset;
    prg[offset] = 0xA9; offset += 1; // LDA #$FF
    prg[offset] = 0xFF; offset += 1;
    let wait_inner = offset;
    prg[offset] = 0x38; offset += 1; // SEC
    prg[offset] = 0xE9; offset += 1; // SBC #$01
    prg[offset] = 0x01; offset += 1;
    prg[offset] = 0xD0; offset += 1; // BNE wait_inner
    prg[offset] = (wait_inner as i32 - offset as i32 - 1) as u8; offset += 1;
    prg[offset] = 0x88; offset += 1; // DEY
    prg[offset] = 0xD0; offset += 1; // BNE wait_outer
    prg[offset] = (wait_outer as i32 - offset as i32 - 1) as u8; offset += 1;
    
    // Next note
    prg[offset] = 0xE8; offset += 1; // INX
    prg[offset] = 0x4C; offset += 1; // JMP play_loop
    prg[offset] = (play_loop & 0xFF) as u8; offset += 1;
    prg[offset] = ((play_loop >> 8) | 0xC0) as u8; offset += 1;
    
    // Restart melody
    prg[offset] = 0x4C; offset += 1; // JMP melody_start
    prg[offset] = (melody_start & 0xFF) as u8; offset += 1;
    prg[offset] = ((melody_start >> 8) | 0xC0) as u8; offset += 1;
    
    // Add note table at $C200 (0x200 in PRG ROM)
    let note_table_offset = 0x200;
    let notes = [
        0x42, // C4
        0x3C, // D4
        0x35, // E4
        0x31, // F4
        0x2C, // G4
        0x35, // E4
        0x42, // C4
        0x00, // End marker
    ];
    for (i, &note) in notes.iter().enumerate() {
        prg[note_table_offset + i] = note;
    }
    
    // Set reset vector
    prg[0x3FFC] = 0x00;  // Low byte
    prg[0x3FFD] = 0xC0;  // High byte ($C000)
    
    rom.extend_from_slice(&prg);
    
    rom
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccnes_core::{Cartridge, Nes};
    use std::io::Cursor;
    
    #[test]
    fn test_audio_demo_rom() {
        let rom_data = create_audio_demo_rom();
        let cartridge = Cartridge::from_ines(Cursor::new(rom_data)).unwrap();
        
        let mut nes = Nes::new();
        nes.load_cartridge(cartridge);
        
        // Run for several frames
        for frame in 0..120 {
            nes.run_frame();
            
            let samples = nes.bus.apu.get_samples();
            if frame > 5 { // Give it time to initialize
                assert!(!samples.is_empty(), "Should generate audio samples");
            }
        }
    }
}

// Generate the ROM file
#[test]
#[ignore] // Run with --ignored to generate the file
fn generate_audio_demo_file() {
    let rom_data = create_audio_demo_rom();
    let mut file = File::create("audio_demo.nes").unwrap();
    file.write_all(&rom_data).unwrap();
    println!("Generated audio_demo.nes");
}
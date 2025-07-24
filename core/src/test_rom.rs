// Simple test ROM for basic functionality testing

pub fn create_test_rom() -> Vec<u8> {
    let mut rom = Vec::new();
    
    // iNES header
    rom.extend_from_slice(b"NES\x1A");  // Magic
    rom.push(1);  // 1x 16KB PRG ROM
    rom.push(1);  // 1x 8KB CHR ROM
    rom.push(0);  // Mapper 0, horizontal mirroring
    rom.push(0);  // Mapper 0 high nibble
    rom.extend_from_slice(&[0; 8]);  // Padding
    
    // PRG ROM (16KB)
    let mut prg = vec![0; 16384];
    
    // Simple test program at reset vector
    // This program sets some colors and loops forever
    let program = [
        0xA9, 0x00,  // LDA #$00
        0x8D, 0x00, 0x20,  // STA $2000 (PPUCTRL = 0)
        0x8D, 0x01, 0x20,  // STA $2001 (PPUMASK = 0)
        
        // Wait for PPU to stabilize (2 vblanks)
        0x2C, 0x02, 0x20,  // BIT $2002
        0x10, 0xFB,        // BPL -5
        0x2C, 0x02, 0x20,  // BIT $2002
        0x10, 0xFB,        // BPL -5
        
        // Set palette
        0xA9, 0x3F,        // LDA #$3F
        0x8D, 0x06, 0x20,  // STA $2006
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x06, 0x20,  // STA $2006
        
        // Background color (light blue)
        0xA9, 0x21,        // LDA #$21
        0x8D, 0x07, 0x20,  // STA $2007
        
        // Fill nametable with pattern 0
        0xA9, 0x20,        // LDA #$20
        0x8D, 0x06, 0x20,  // STA $2006
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x06, 0x20,  // STA $2006
        
        0xA9, 0x00,        // LDA #$00 (tile 0)
        0xA2, 0x00,        // LDX #$00
        0xA0, 0x04,        // LDY #$04 (4 pages = 1024 bytes)
        
        // Fill loop
        0x8D, 0x07, 0x20,  // STA $2007
        0xE8,              // INX
        0xD0, 0xFA,        // BNE -6
        0x88,              // DEY
        0xD0, 0xF7,        // BNE -9
        
        // Enable rendering
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x05, 0x20,  // STA $2005 (scroll X = 0)
        0x8D, 0x05, 0x20,  // STA $2005 (scroll Y = 0)
        0xA9, 0x08,        // LDA #$08
        0x8D, 0x01, 0x20,  // STA $2001 (show background)
        0xA9, 0x80,        // LDA #$80
        0x8D, 0x00, 0x20,  // STA $2000 (enable NMI)
        
        // Infinite loop
        0x4C, 0x5A, 0xC0,  // JMP $C05A
    ];
    
    // Place program at start of ROM
    for (i, &byte) in program.iter().enumerate() {
        prg[i] = byte;
    }
    
    // Set reset vector
    prg[0x3FFC] = 0x00;  // Low byte
    prg[0x3FFD] = 0xC0;  // High byte ($C000)
    
    rom.extend_from_slice(&prg);
    
    // CHR ROM (8KB) - Simple pattern
    let mut chr = vec![0; 8192];
    
    // Create a simple tile pattern (checkerboard)
    for i in 0..8 {
        chr[i] = if i % 2 == 0 { 0xAA } else { 0x55 };
        chr[i + 8] = if i % 2 == 0 { 0xAA } else { 0x55 };
    }
    
    rom.extend_from_slice(&chr);
    
    rom
}
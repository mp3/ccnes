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

pub fn create_sprite_test_rom() -> Vec<u8> {
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
    
    // Program that displays sprites
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
        
        // Background palette
        0xA9, 0x0F,        // LDA #$0F (black)
        0x8D, 0x07, 0x20,  // STA $2007
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x07, 0x20,  // STA $2007
        0xA9, 0x10,        // LDA #$10
        0x8D, 0x07, 0x20,  // STA $2007
        0xA9, 0x30,        // LDA #$30
        0x8D, 0x07, 0x20,  // STA $2007
        
        // Sprite palette 0
        0xA9, 0x3F,        // LDA #$3F
        0x8D, 0x06, 0x20,  // STA $2006
        0xA9, 0x10,        // LDA #$10
        0x8D, 0x06, 0x20,  // STA $2006
        
        0xA9, 0x0F,        // LDA #$0F (black)
        0x8D, 0x07, 0x20,  // STA $2007
        0xA9, 0x16,        // LDA #$16 (red)
        0x8D, 0x07, 0x20,  // STA $2007
        0xA9, 0x27,        // LDA #$27 (orange)
        0x8D, 0x07, 0x20,  // STA $2007
        0xA9, 0x18,        // LDA #$18 (yellow)
        0x8D, 0x07, 0x20,  // STA $2007
        
        // Clear OAM
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x03, 0x20,  // STA $2003 (OAMADDR = 0)
        0xA2, 0x00,        // LDX #$00
        0xA9, 0xFF,        // LDA #$FF
        // Clear OAM loop
        0x8D, 0x04, 0x20,  // STA $2004
        0xE8,              // INX
        0xD0, 0xFA,        // BNE -6
        
        // Set up sprite 0 at (128, 120)
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x03, 0x20,  // STA $2003 (OAMADDR = 0)
        
        0xA9, 0x78,        // LDA #$78 (Y = 120)
        0x8D, 0x04, 0x20,  // STA $2004
        0xA9, 0x01,        // LDA #$01 (tile 1)
        0x8D, 0x04, 0x20,  // STA $2004
        0xA9, 0x00,        // LDA #$00 (attributes)
        0x8D, 0x04, 0x20,  // STA $2004
        0xA9, 0x80,        // LDA #$80 (X = 128)
        0x8D, 0x04, 0x20,  // STA $2004
        
        // Enable rendering
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x05, 0x20,  // STA $2005 (scroll X = 0)
        0x8D, 0x05, 0x20,  // STA $2005 (scroll Y = 0)
        0xA9, 0x18,        // LDA #$18
        0x8D, 0x01, 0x20,  // STA $2001 (show background + sprites)
        0xA9, 0x80,        // LDA #$80
        0x8D, 0x00, 0x20,  // STA $2000 (enable NMI)
        
        // Infinite loop
        0x4C, 0x80, 0xC0,  // JMP $C080
    ];
    
    // Place program at start of ROM
    for (i, &byte) in program.iter().enumerate() {
        prg[i] = byte;
    }
    
    // Set reset vector
    prg[0x3FFC] = 0x00;  // Low byte
    prg[0x3FFD] = 0xC0;  // High byte ($C000)
    
    rom.extend_from_slice(&prg);
    
    // CHR ROM (8KB)
    let mut chr = vec![0; 8192];
    
    // Tile 0: Background (empty)
    // Already zeroed
    
    // Tile 1: Sprite (smiley face)
    let sprite_pattern = [
        0b00111100,
        0b01000010,
        0b10100101,
        0b10000001,
        0b10100101,
        0b10011001,
        0b01000010,
        0b00111100,
    ];
    
    // Low byte plane
    for (i, &byte) in sprite_pattern.iter().enumerate() {
        chr[0x10 + i] = byte;
    }
    // High byte plane (same pattern for simplicity)
    for (i, &byte) in sprite_pattern.iter().enumerate() {
        chr[0x18 + i] = byte;
    }
    
    rom.extend_from_slice(&chr);
    
    rom
}

pub fn create_controller_test_rom() -> Vec<u8> {
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
    
    // Program that reads controller and stores in RAM
    let program = [
        // Initialize
        0xA9, 0x00,        // LDA #$00
        0x85, 0x00,        // STA $00 (clear result)
        
        // Strobe controller
        0xA9, 0x01,        // LDA #$01
        0x8D, 0x16, 0x40,  // STA $4016
        0xA9, 0x00,        // LDA #$00
        0x8D, 0x16, 0x40,  // STA $4016
        
        // Read 8 bits from controller
        0xA2, 0x08,        // LDX #$08
        // Read loop (at $C00F)
        0xAD, 0x16, 0x40,  // LDA $4016
        0x4A,              // LSR A
        0x66, 0x00,        // ROR $00
        0xCA,              // DEX
        0xD0, 0xF6,        // BNE -10 (back to $C00F)
        
        // Infinite loop
        0x4C, 0x18, 0xC0,  // JMP $C018
    ];
    
    // Place program at start of ROM
    for (i, &byte) in program.iter().enumerate() {
        prg[i] = byte;
    }
    
    // Set reset vector
    prg[0x3FFC] = 0x00;  // Low byte
    prg[0x3FFD] = 0xC0;  // High byte ($C000)
    
    rom.extend_from_slice(&prg);
    
    // CHR ROM (8KB) - empty
    let chr = vec![0; 8192];
    rom.extend_from_slice(&chr);
    
    rom
}
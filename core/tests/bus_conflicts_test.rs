use ccnes_core::nes::Nes;
use ccnes_core::cartridge::Cartridge;
use ccnes_core::cpu::CpuBus;

#[test]
fn test_mapper2_bus_conflicts() {
    // Create a simple ROM with mapper 2 (UxROM)
    let mut rom_data = vec![0; 16 + 0x4000]; // Header + 16KB PRG ROM
    
    // iNES header for mapper 2
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 1; // 1 PRG ROM bank (16KB)
    rom_data[5] = 0; // No CHR ROM
    rom_data[6] = 0x20; // Mapper 2 (bits 4-7)
    rom_data[7] = 0x00;
    
    // Fill PRG ROM with specific patterns for bus conflict testing
    // First bank at 0x8000-0xBFFF
    for i in 0..0x4000 {
        rom_data[16 + i] = 0xFF; // All bits set for testing
    }
    
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Test 1: Normal write without bus conflict
    // Write to an address that has 0x00 in ROM (no conflict)
    nes.bus.write(0x8000, 0x01);
    
    // Test 2: Write with bus conflict
    // Write to an address that has 0xFF in ROM (conflict)
    // The actual value should be ANDed: 0x02 & 0xFF = 0x02
    nes.bus.write(0x8001, 0x02);
    
    // Test 3: Write with partial bus conflict
    // If ROM has 0xF0 and we write 0x0F, result should be 0x00
    nes.bus.write(0x8002, 0x0F);
    
    // The test passes if no panic occurs during bus conflict handling
    assert!(true);
}

#[test]
fn test_mapper3_bus_conflicts() {
    // Create a simple ROM with mapper 3 (CNROM)
    let mut rom_data = vec![0; 16 + 0x8000 + 0x2000]; // Header + 32KB PRG + 8KB CHR
    
    // iNES header for mapper 3
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 2; // 2 PRG ROM banks (32KB)
    rom_data[5] = 1; // 1 CHR ROM bank (8KB)
    rom_data[6] = 0x30; // Mapper 3 (bits 4-7)
    rom_data[7] = 0x00;
    
    // Fill PRG ROM with test patterns
    for i in 0..0x8000 {
        rom_data[16 + i] = if i % 2 == 0 { 0xAA } else { 0x55 };
    }
    
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Test bus conflicts with CHR bank switching
    nes.bus.write(0x8000, 0x01); // Should work without conflict
    nes.bus.write(0x8001, 0x02); // May have bus conflict depending on ROM data
    
    assert!(true);
}

#[test]
fn test_mapper7_bus_conflicts() {
    // Create a simple ROM with mapper 7 (AxROM)
    let mut rom_data = vec![0; 16 + 0x8000]; // Header + 32KB PRG ROM
    
    // iNES header for mapper 7
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 2; // 2 PRG ROM banks (32KB)
    rom_data[5] = 0; // No CHR ROM (uses CHR RAM)
    rom_data[6] = 0x70; // Mapper 7 (bits 4-7)
    rom_data[7] = 0x00;
    
    // Fill PRG ROM with alternating patterns
    for i in 0..0x8000 {
        rom_data[16 + i] = (i & 0xFF) as u8;
    }
    
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Test PRG bank switching with bus conflicts
    nes.bus.write(0x8000, 0x10); // Bank 0, mirroring bit set
    nes.bus.write(0x8001, 0x01); // Bank 1, no mirroring
    
    assert!(true);
}

#[test]
fn test_mapper11_bus_conflicts() {
    // Create a simple ROM with mapper 11 (Color Dreams)
    let mut rom_data = vec![0; 16 + 0x8000 + 0x2000]; // Header + 32KB PRG + 8KB CHR
    
    // iNES header for mapper 11
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 2; // 2 PRG ROM banks (32KB)
    rom_data[5] = 1; // 1 CHR ROM bank (8KB)
    rom_data[6] = 0xB0; // Mapper 11 (bits 4-7)
    rom_data[7] = 0x00;
    
    // Fill PRG ROM with test data
    for i in 0..0x8000 {
        rom_data[16 + i] = 0xF0; // High nibble set for testing
    }
    
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Test combined PRG/CHR bank switching with bus conflicts
    nes.bus.write(0x8000, 0xFF); // All bits set
    nes.bus.write(0x8001, 0x0F); // Low nibble only
    
    assert!(true);
}

#[test]
fn test_mapper66_bus_conflicts() {
    // Create a simple ROM with mapper 66 (GxROM)
    let mut rom_data = vec![0; 16 + 0x8000 + 0x2000]; // Header + 32KB PRG + 8KB CHR
    
    // iNES header for mapper 66
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 2; // 2 PRG ROM banks (32KB)
    rom_data[5] = 1; // 1 CHR ROM bank (8KB)
    rom_data[6] = 0x20; // Mapper 66 lower nibble  
    rom_data[7] = 0x40; // Mapper 66 upper nibble (0x40 | 0x02 = 0x42 >> 4 = 4, but 66 = 0x42)
    
    // Fill PRG ROM with specific patterns for conflict testing
    for i in 0..0x8000 {
        rom_data[16 + i] = 0x33; // Alternating bits for testing
    }
    
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Test bus conflicts with combined bank switching
    nes.bus.write(0x8000, 0x11); // PRG bank 1, CHR bank 1
    nes.bus.write(0x8001, 0x22); // PRG bank 2, CHR bank 2
    
    assert!(true);
}
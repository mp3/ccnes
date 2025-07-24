use ccnes_core::{Cartridge, Nes};

#[test]
fn test_sprite_rendering() {
    // Create sprite test ROM
    let rom_data = ccnes_core::test_rom::create_sprite_test_rom();
    
    // Load into cartridge
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load sprite test ROM");
    
    // Create NES and load cartridge
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Run for several frames to ensure sprites are rendered
    for _ in 0..10 {
        nes.run_frame();
    }
    
    // Check that we have non-zero pixels (sprite should be visible)
    let framebuffer = nes.get_framebuffer();
    let non_zero_pixels = framebuffer.iter().filter(|&&p| p != 0).count();
    
    assert!(non_zero_pixels > 0, "Framebuffer should have non-zero pixels");
    
    // Check that we have more than just background color
    // The sprite should create different colored pixels
    let unique_colors: std::collections::HashSet<_> = framebuffer.iter().collect();
    assert!(unique_colors.len() > 1, "Should have multiple colors from sprite");
    
    // Check PPU status
    let ppu_mask = nes.bus.ppu.read_register(1);
    assert_eq!(ppu_mask & 0x18, 0x18, "Both background and sprites should be enabled");
}

#[test]
fn test_oam_dma() {
    use ccnes_core::cpu::CpuBus;
    
    // Create a simple test
    let rom_data = ccnes_core::test_rom::create_sprite_test_rom();
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load ROM");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Set up some data in RAM page 2
    for i in 0..256 {
        nes.bus.write(0x0200 + i, i as u8);
    }
    
    // Trigger OAM DMA from page 2
    nes.bus.write(0x4014, 0x02);
    
    // Run enough cycles for DMA to complete (513+ cycles)
    for _ in 0..600 {
        nes.step();
    }
    
    // Verify OAM was filled with data from page 2
    for i in 0..256 {
        let oam_value = nes.bus.ppu.oam[i];
        assert_eq!(oam_value, i as u8, "OAM[{}] should equal {}", i, i);
    }
}
use ccnes_core::{Cartridge, Nes};

#[test]
fn debug_sprite_rendering() {
    // Create sprite test ROM
    let rom_data = ccnes_core::test_rom::create_sprite_test_rom();
    
    // Load into cartridge
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load sprite test ROM");
    
    // Create NES and load cartridge
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    println!("Initial state:");
    println!("PPU CTRL: {:02X}", nes.bus.ppu.get_ctrl());
    println!("PPU MASK: {:02X}", nes.bus.ppu.read_register(1));
    
    // Run for a few frames
    for frame in 0..5 {
        println!("\nFrame {}:", frame);
        nes.run_frame();
        
        // Check PPU state
        println!("PPU CTRL: {:02X}", nes.bus.ppu.get_ctrl());
        println!("PPU MASK: {:02X}", nes.bus.ppu.read_register(1));
        println!("PPU STATUS: {:02X}", nes.bus.ppu.read_register(2));
        
        // Check some pixels
        let framebuffer = nes.get_framebuffer();
        let non_zero = framebuffer.iter().filter(|&&p| p != 0).count();
        println!("Non-zero pixels: {}", non_zero);
        
        // Check palette entries
        println!("Palette[0]: {:02X}", nes.bus.ppu.palette[0]);
        println!("Palette[16]: {:02X}", nes.bus.ppu.palette[16]);
        println!("Palette[17-19]: {:02X} {:02X} {:02X}", 
            nes.bus.ppu.palette[17], nes.bus.ppu.palette[18], nes.bus.ppu.palette[19]);
        
        // Check OAM
        println!("OAM[0-3]: {:02X} {:02X} {:02X} {:02X}", 
            nes.bus.ppu.oam[0], nes.bus.ppu.oam[1], 
            nes.bus.ppu.oam[2], nes.bus.ppu.oam[3]);
    }
}
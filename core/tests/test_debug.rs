use ccnes_core::{Cartridge, Nes};

#[test]
fn debug_test_rom() {
    // Create test ROM
    let rom_data = ccnes_core::test_rom::create_test_rom();
    
    // Load into cartridge
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load test ROM");
    
    // Create NES and load cartridge
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Step through a few instructions
    println!("Starting CPU execution...");
    for i in 0..20 {
        let pc = nes.cpu.pc;
        let cycles = nes.cpu.cycles;
        println!("Step {}: PC={:04X}, cycles={}", i, pc, cycles);
        
        // Step one CPU instruction
        nes.step();
        
        // Check if we hit an invalid opcode
        if i > 0 && nes.cpu.pc == pc {
            println!("CPU seems stuck at PC={:04X}", pc);
            break;
        }
    }
}
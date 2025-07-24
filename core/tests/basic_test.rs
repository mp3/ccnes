use ccnes_core::{Cartridge, Nes};

#[test]
fn test_basic_rom_execution() {
    // Create test ROM
    let rom_data = ccnes_core::test_rom::create_test_rom();
    
    // Load into cartridge
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load test ROM");
    
    // Create NES and load cartridge
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Run for a few frames
    for _ in 0..5 {
        nes.run_frame();
    }
    
    // Check that PPU has non-zero framebuffer (background color set)
    let framebuffer = nes.get_framebuffer();
    let non_zero_pixels = framebuffer.iter().filter(|&&p| p != 0).count();
    
    assert!(non_zero_pixels > 0, "Framebuffer should have non-zero pixels");
    
    // Check that CPU executed some instructions
    assert!(nes.cpu.cycles > 1000, "CPU should have executed many cycles");
    
    // Check that PPU is rendering
    assert_eq!(nes.bus.ppu.get_ctrl() & 0x80, 0x80, "NMI should be enabled");
}

#[test] 
fn test_cpu_basic_instructions() {
    use ccnes_core::Cpu;
    use ccnes_core::cpu::CpuBus;
    
    struct TestBus {
        memory: [u8; 0x10000],
    }
    
    impl CpuBus for TestBus {
        fn read(&mut self, addr: u16) -> u8 {
            self.memory[addr as usize]
        }
        
        fn write(&mut self, addr: u16, value: u8) {
            self.memory[addr as usize] = value;
        }
    }
    
    let mut cpu = Cpu::new();
    let mut bus = TestBus { memory: [0; 0x10000] };
    
    // Set reset vector
    bus.memory[0xFFFC] = 0x00;
    bus.memory[0xFFFD] = 0xC0;
    
    // Test LDA immediate
    bus.memory[0xC000] = 0xA9;  // LDA #$42
    bus.memory[0xC001] = 0x42;
    
    // Test STA zero page
    bus.memory[0xC002] = 0x85;  // STA $00
    bus.memory[0xC003] = 0x00;
    
    // Test NOP
    bus.memory[0xC004] = 0xEA;  // NOP
    
    // Initialize CPU
    cpu.reset(&mut bus);
    assert_eq!(cpu.pc, 0xC000);
    
    // Execute LDA
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x42);
    
    // Execute STA
    cpu.step(&mut bus);
    assert_eq!(bus.memory[0], 0x42);
    
    // Execute NOP
    let cycles_before = cpu.cycles;
    cpu.step(&mut bus);
    assert_eq!(cpu.cycles - cycles_before, 2);
}
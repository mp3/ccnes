use ccnes_core::{Cartridge, Nes, Controller, ControllerButton};
use ccnes_core::cpu::CpuBus;

#[test]
fn test_controller_input() {
    // Create a test ROM that reads controller input
    let rom_data = ccnes_core::test_rom::create_controller_test_rom();
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load ROM");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Set up controller state
    let mut controller = Controller::new();
    controller.set_button(ControllerButton::A, true);
    controller.set_button(ControllerButton::UP, true);
    
    // Apply controller state to NES
    nes.set_controller1_from_controller(&controller);
    
    // Run for a bit to let the ROM read controller
    for _ in 0..100 {
        nes.step();
    }
    
    // The test ROM should store controller state in RAM
    let controller_value = nes.bus.read(0x00);
    assert_eq!(controller_value, ControllerButton::A.bits() | ControllerButton::UP.bits());
}

#[test]
fn test_controller_strobe() {
    let rom_data = ccnes_core::test_rom::create_test_rom();
    let cartridge = Cartridge::from_ines(std::io::Cursor::new(rom_data))
        .expect("Failed to load ROM");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Set controller state
    nes.set_controller1(0xFF); // All buttons pressed
    
    // Write strobe
    nes.bus.write(0x4016, 0x01);
    nes.bus.write(0x4016, 0x00);
    
    // Read all 8 bits
    let mut result = 0;
    for i in 0..8 {
        let bit = nes.bus.read(0x4016) & 0x01;
        result |= bit << i;
    }
    
    assert_eq!(result, 0xFF);
}

#[test]
fn test_controller_api() {
    let mut controller = Controller::new();
    
    // Test setting individual buttons
    controller.set_button(ControllerButton::START, true);
    controller.set_button(ControllerButton::SELECT, true);
    assert_eq!(controller.get_state(), 0x0C);
    
    // Test setting multiple buttons at once
    controller.set_buttons(ControllerButton::A | ControllerButton::B | ControllerButton::RIGHT);
    assert_eq!(controller.get_state(), 0x83);
    
    // Test checking individual buttons
    assert!(controller.is_pressed(ControllerButton::A));
    assert!(controller.is_pressed(ControllerButton::B));
    assert!(controller.is_pressed(ControllerButton::RIGHT));
    assert!(!controller.is_pressed(ControllerButton::LEFT));
    
    // Test clearing
    controller.clear();
    assert_eq!(controller.get_state(), 0x00);
}
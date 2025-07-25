use ccnes_core::nes::Nes;
use ccnes_core::cartridge::Cartridge;
use ccnes_core::savestate::{SaveStateManager, SaveStateError};
use std::io::Cursor;
use tempfile::TempDir;

fn create_test_rom() -> Vec<u8> {
    let mut rom_data = vec![0; 16 + 0x4000]; // Header + 16KB PRG ROM
    
    // iNES header for mapper 0
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 1; // 1 PRG ROM bank (16KB)
    rom_data[5] = 0; // No CHR ROM
    rom_data[6] = 0x00; // Mapper 0
    rom_data[7] = 0x00;
    
    // Simple test program
    rom_data[16 + 0x3FFC] = 0x00; // Reset vector low
    rom_data[16 + 0x3FFD] = 0x80; // Reset vector high
    rom_data[16] = 0xA9; // LDA #$42
    rom_data[17] = 0x42;
    rom_data[18] = 0x8D; // STA $0200
    rom_data[19] = 0x00;
    rom_data[20] = 0x02;
    rom_data[21] = 0x4C; // JMP $8000 (infinite loop)
    rom_data[22] = 0x00;
    rom_data[23] = 0x80;
    
    rom_data
}

#[test]
fn test_savestate_basic() {
    let rom_data = create_test_rom();
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Run a few steps
    for _ in 0..10 {
        nes.step();
    }
    
    // Create save state
    let save_data = nes.save_state_to_vec().expect("Failed to save state");
    
    // Modify NES state
    for _ in 0..10 {
        nes.step();
    }
    
    // Load save state
    nes.load_state_from_slice(&save_data).expect("Failed to load state");
    
    // Test passes if no errors occur
    assert!(true);
}

#[test]
fn test_savestate_serialization() {
    let rom_data = create_test_rom();
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Run some steps to change state
    for _ in 0..50 {
        nes.step();
    }
    
    // Save to memory
    let mut buffer = Vec::new();
    nes.save_state(&mut buffer).expect("Failed to save state");
    
    // Load from memory
    let mut cursor = Cursor::new(&buffer);
    nes.load_state(&mut cursor).expect("Failed to load state");
    
    assert!(true);
}

#[test]
fn test_savestate_manager() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path().join("test_game");
    let manager = SaveStateManager::new(&base_path, 10);
    
    let rom_data = create_test_rom();
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Run some steps
    for _ in 0..20 {
        nes.step();
    }
    
    // Save to slot 0
    manager.save_slot(&nes, 0).expect("Failed to save to slot 0");
    assert!(manager.slot_exists(0));
    
    // Modify state
    for _ in 0..20 {
        nes.step();
    }
    
    // Save to slot 1
    manager.save_slot(&nes, 1).expect("Failed to save to slot 1");
    assert!(manager.slot_exists(1));
    
    // Load from slot 0
    manager.load_slot(&mut nes, 0).expect("Failed to load from slot 0");
    
    // Check that both slots exist
    let existing_slots = manager.list_existing_slots();
    assert!(existing_slots.contains(&0));
    assert!(existing_slots.contains(&1));
    assert_eq!(existing_slots.len(), 2);
    
    // Delete slot 1
    manager.delete_slot(1).expect("Failed to delete slot 1");
    assert!(!manager.slot_exists(1));
    
    // Only slot 0 should exist now
    let existing_slots = manager.list_existing_slots();
    assert_eq!(existing_slots, vec![0]);
}

#[test]
fn test_savestate_manager_invalid_slot() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path().join("test_game");
    let manager = SaveStateManager::new(&base_path, 5); // Only 5 slots
    
    let rom_data = create_test_rom();
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Try to save to invalid slot
    let result = manager.save_slot(&nes, 10);
    assert!(result.is_err());
    
    // Try to load from non-existent slot
    let result = manager.load_slot(&mut nes, 0);
    assert!(result.is_err());
}

#[test]
fn test_savestate_version_validation() {
    let rom_data = create_test_rom();
    let cartridge = Cartridge::from_ines(&rom_data[..]).expect("Failed to create cartridge");
    
    let mut nes = Nes::new();
    nes.load_cartridge(cartridge);
    
    // Create a save state
    let save_data = nes.save_state_to_vec().expect("Failed to save state");
    
    // Corrupt the magic bytes (first 4 bytes)
    let mut corrupted_data = save_data.clone();
    corrupted_data[0] = 0xFF;
    
    // Should fail to load corrupted data
    let result = nes.load_state_from_slice(&corrupted_data);
    assert!(result.is_err());
    
    // Original data should still work
    let result = nes.load_state_from_slice(&save_data);
    assert!(result.is_ok());
}
use serde::{Serialize, Deserialize};
use crate::{Cpu, Bus};
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SaveStateError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid save state version")]
    InvalidVersion,
}

const SAVE_STATE_VERSION: u32 = 2;
const SAVE_STATE_MAGIC: &[u8; 4] = b"CCNS";

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    magic: [u8; 4],
    version: u32,
    
    // CPU state
    cpu_a: u8,
    cpu_x: u8,
    cpu_y: u8,
    cpu_sp: u8,
    cpu_pc: u16,
    cpu_status: u8,
    cpu_cycles: u32,
    
    // Essential PPU state (publicly accessible)
    ppu_palette: Vec<u8>,
    ppu_oam: Vec<u8>,
    
    // Essential memory state
    ram: Vec<u8>,
    
    // Mapper state
    mapper_number: u8,
    mapper_state: crate::cartridge::MapperState,
    mirroring: crate::cartridge::Mirroring,
    
    // Controller state
    controller1_state: u8,
    controller2_state: u8,
}

impl SaveState {
    pub fn create_quick_save(cpu: &Cpu, bus: &Bus) -> Self {
        SaveState {
            magic: *SAVE_STATE_MAGIC,
            version: SAVE_STATE_VERSION,
            
            // CPU state
            cpu_a: cpu.a,
            cpu_x: cpu.x,
            cpu_y: cpu.y,
            cpu_sp: cpu.sp,
            cpu_pc: cpu.pc,
            cpu_status: cpu.status.bits(),
            cpu_cycles: cpu.cycles,
            
            // Essential PPU state
            ppu_palette: bus.ppu.palette.to_vec(),
            ppu_oam: bus.ppu.oam.to_vec(),
            
            // Essential memory state
            ram: bus.get_ram().to_vec(),
            
            // Mapper state
            mapper_number: bus.cartridge.as_ref().map_or(255, |c| c.get_mapper_number()),
            mapper_state: bus.cartridge.as_ref().map_or(crate::cartridge::MapperState::Other, |c| c.get_mapper_state()),
            mirroring: bus.cartridge.as_ref().map_or(crate::cartridge::Mirroring::Horizontal, |c| c.mirroring()),
            
            // Controller state
            controller1_state: bus.get_controller1_state(),
            controller2_state: bus.get_controller2_state(),
        }
    }
    
    pub fn restore_quick_save(&self, cpu: &mut Cpu, bus: &mut Bus) -> Result<(), SaveStateError> {
        // Verify magic and version
        if &self.magic != SAVE_STATE_MAGIC {
            return Err(SaveStateError::InvalidVersion);
        }
        if self.version != SAVE_STATE_VERSION {
            return Err(SaveStateError::InvalidVersion);
        }
        
        // Restore CPU state
        cpu.a = self.cpu_a;
        cpu.x = self.cpu_x;
        cpu.y = self.cpu_y;
        cpu.sp = self.cpu_sp;
        cpu.pc = self.cpu_pc;
        cpu.status = crate::cpu::StatusFlags::from_bits_truncate(self.cpu_status);
        cpu.cycles = self.cpu_cycles;
        
        // Restore essential PPU state
        bus.ppu.palette.copy_from_slice(&self.ppu_palette);
        bus.ppu.oam.copy_from_slice(&self.ppu_oam);
        
        // Restore memory state
        bus.set_ram(&self.ram);
        
        // Restore mapper state
        if let Some(cartridge) = &mut bus.cartridge {
            cartridge.set_mapper_state(&self.mapper_state);
        }
        
        // Restore controller state
        bus.set_controller_states(self.controller1_state, self.controller2_state);
        
        Ok(())
    }
    
    pub fn save<W: Write>(&self, writer: W) -> Result<(), SaveStateError> {
        bincode::serialize_into(writer, self)?;
        Ok(())
    }
    
    pub fn load<R: Read>(reader: R) -> Result<Self, SaveStateError> {
        let state = bincode::deserialize_from(reader)?;
        Ok(state)
    }
}

// Helper methods for Nes struct
use crate::Nes;

impl Nes {
    /// Create a quick save state (captures essential state only)
    pub fn quick_save(&self) -> SaveState {
        SaveState::create_quick_save(&self.cpu, &self.bus)
    }
    
    /// Restore from a quick save state
    pub fn quick_load(&mut self, state: &SaveState) -> Result<(), SaveStateError> {
        state.restore_quick_save(&mut self.cpu, &mut self.bus)
    }
    
    /// Save state to a writer
    pub fn save_state<W: Write>(&self, writer: W) -> Result<(), SaveStateError> {
        let state = self.quick_save();
        state.save(writer)
    }
    
    /// Load state from a reader
    pub fn load_state<R: Read>(&mut self, reader: R) -> Result<(), SaveStateError> {
        let state = SaveState::load(reader)?;
        self.quick_load(&state)
    }
    
    /// Save state to a byte vector
    pub fn save_state_to_vec(&self) -> Result<Vec<u8>, SaveStateError> {
        let mut data = Vec::new();
        self.save_state(&mut data)?;
        Ok(data)
    }
    
    /// Load state from a byte slice
    pub fn load_state_from_slice(&mut self, data: &[u8]) -> Result<(), SaveStateError> {
        self.load_state(std::io::Cursor::new(data))
    }
}

// Save state slot management
use std::path::Path;
use std::fs;

pub struct SaveStateManager {
    base_path: std::path::PathBuf,
    max_slots: usize,
}

impl SaveStateManager {
    pub fn new<P: AsRef<Path>>(base_path: P, max_slots: usize) -> Self {
        let base_path = base_path.as_ref().to_path_buf();
        // Create directory if it doesn't exist
        if let Some(parent) = base_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        Self {
            base_path,
            max_slots,
        }
    }
    
    /// Save to a specific slot
    pub fn save_slot(&self, nes: &Nes, slot: usize) -> Result<(), SaveStateError> {
        if slot >= self.max_slots {
            return Err(SaveStateError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Slot {} exceeds maximum slots {}", slot, self.max_slots)
            )));
        }
        
        let path = self.get_slot_path(slot);
        let file = fs::File::create(path)?;
        nes.save_state(file)
    }
    
    /// Load from a specific slot
    pub fn load_slot(&self, nes: &mut Nes, slot: usize) -> Result<(), SaveStateError> {
        if slot >= self.max_slots {
            return Err(SaveStateError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Slot {} exceeds maximum slots {}", slot, self.max_slots)
            )));
        }
        
        let path = self.get_slot_path(slot);
        if !path.exists() {
            return Err(SaveStateError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Save slot {} does not exist", slot)
            )));
        }
        
        let file = fs::File::open(path)?;
        nes.load_state(file)
    }
    
    /// Check if a slot has a save state
    pub fn slot_exists(&self, slot: usize) -> bool {
        if slot >= self.max_slots {
            return false;
        }
        self.get_slot_path(slot).exists()
    }
    
    /// Delete a save state slot
    pub fn delete_slot(&self, slot: usize) -> Result<(), SaveStateError> {
        if slot >= self.max_slots {
            return Ok(()); // Slot doesn't exist anyway
        }
        
        let path = self.get_slot_path(slot);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
    
    /// Get metadata about a save slot (creation time, etc.)
    pub fn get_slot_metadata(&self, slot: usize) -> Option<std::fs::Metadata> {
        if slot >= self.max_slots {
            return None;
        }
        
        let path = self.get_slot_path(slot);
        fs::metadata(path).ok()
    }
    
    /// List all existing save slots
    pub fn list_existing_slots(&self) -> Vec<usize> {
        (0..self.max_slots)
            .filter(|&slot| self.slot_exists(slot))
            .collect()
    }
    
    fn get_slot_path(&self, slot: usize) -> std::path::PathBuf {
        let mut path = self.base_path.clone();
        path.set_extension(format!("slot{}.ccnes", slot));
        path
    }
    
    /// Get the base filename for save states
    pub fn get_base_name(&self) -> Option<&str> {
        self.base_path.file_stem().and_then(|s| s.to_str())
    }
}
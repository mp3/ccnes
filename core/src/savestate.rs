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

const SAVE_STATE_VERSION: u32 = 1;
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
    
    // Essential memory state
    ram: Vec<u8>,
    ppu_palette: Vec<u8>,
    ppu_oam: Vec<u8>,
    
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
            
            // Essential memory state
            ram: bus.get_ram().to_vec(),
            ppu_palette: bus.ppu.palette.to_vec(),
            ppu_oam: bus.ppu.oam.to_vec(),
            
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
        
        // Restore memory state
        bus.set_ram(&self.ram);
        bus.ppu.palette.copy_from_slice(&self.ppu_palette);
        bus.ppu.oam.copy_from_slice(&self.ppu_oam);
        
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
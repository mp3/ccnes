use std::io::Read;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub mod mappers;

#[derive(Debug, Error)]
pub enum CartridgeError {
    #[error("Invalid iNES header")]
    InvalidHeader,
    #[error("Unsupported mapper: {0}")]
    UnsupportedMapper(u8),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: Box<dyn Mapper>,
    mirroring: Mirroring,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
    SingleScreenLow,
    SingleScreenHigh,
}

#[derive(Serialize, Deserialize)]
pub enum MapperState {
    Mapper0,
    Mapper1 {
        shift_register: u8,
        shift_count: u8,
        prg_bank_mode: u8,
        chr_bank_mode: u8,
        prg_bank: u8,
        chr_bank0: u8,
        chr_bank1: u8,
    },
    Mapper2 {
        prg_bank: u8,
    },
    Mapper3 {
        chr_bank: u8,
    },
    Mapper4 {
        bank_select: u8,
        prg_mode: bool,
        chr_mode: bool,
        bank_registers: Vec<u8>,
        irq_counter: u8,
        irq_reload: u8,
        irq_enable: bool,
    },
    Mapper7 {
        prg_bank: usize,
        mirroring_mode: u8,
    },
    Other,
}

pub trait Mapper: std::fmt::Debug {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8;
    fn write_prg(&mut self, addr: u16, value: u8);
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8;
    fn write_chr(&mut self, addr: u16, value: u8);
    fn get_state(&self) -> MapperState;
    fn set_state(&mut self, state: &MapperState);
}

impl Cartridge {
    pub fn from_ines<R: Read>(mut reader: R) -> Result<Self, CartridgeError> {
        let mut header = [0u8; 16];
        reader.read_exact(&mut header)?;
        
        // Check "NES\x1A" magic
        if &header[0..4] != b"NES\x1A" {
            return Err(CartridgeError::InvalidHeader);
        }
        
        let prg_size = header[4] as usize * 16384;  // 16KB units
        let chr_size = header[5] as usize * 8192;   // 8KB units
        
        let mapper_num = (header[6] >> 4) | (header[7] & 0xF0);
        
        let mirroring = if header[6] & 0x08 != 0 {
            Mirroring::FourScreen
        } else if header[6] & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        
        // Skip trainer if present
        if header[6] & 0x04 != 0 {
            let mut trainer = [0u8; 512];
            reader.read_exact(&mut trainer)?;
        }
        
        // Read PRG ROM
        let mut prg_rom = vec![0u8; prg_size];
        reader.read_exact(&mut prg_rom)?;
        
        // Read CHR ROM
        let mut chr_rom = vec![0u8; chr_size];
        reader.read_exact(&mut chr_rom)?;
        
        // Create mapper
        let mapper: Box<dyn Mapper> = match mapper_num {
            0 => Box::new(mappers::Mapper0::new()),
            1 => Box::new(mappers::Mapper1::new()),
            2 => Box::new(mappers::Mapper2::new(prg_size)),
            3 => Box::new(mappers::Mapper3::new()),
            4 => Box::new(mappers::Mapper4::new(prg_size, chr_size)),
            5 => Box::new(mappers::Mapper5::new(prg_size, chr_size)),
            7 => Box::new(mappers::Mapper7::new(prg_size)),
            9 => Box::new(mappers::Mapper9::new(prg_size, chr_size)),
            11 => Box::new(mappers::Mapper11::new(prg_size, chr_size)),
            66 => Box::new(mappers::Mapper66::new(prg_size, chr_size)),
            _ => return Err(CartridgeError::UnsupportedMapper(mapper_num)),
        };
        
        Ok(Cartridge {
            prg_rom,
            chr_rom,
            mapper,
            mirroring,
        })
    }
    
    pub fn read_prg(&self, addr: u16) -> u8 {
        self.mapper.read_prg(addr, &self.prg_rom)
    }
    
    pub fn write_prg(&mut self, addr: u16, value: u8) {
        self.mapper.write_prg(addr, value);
    }
    
    pub fn read_chr(&self, addr: u16) -> u8 {
        self.mapper.read_chr(addr, &self.chr_rom)
    }
    
    pub fn write_chr(&mut self, addr: u16, value: u8) {
        self.mapper.write_chr(addr, value);
    }
    
    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    
    pub fn get_mapper_number(&self) -> u8 {
        // This should be stored when creating the cartridge
        // For now, we'll determine from the mapper type
        match self.mapper.get_state() {
            MapperState::Mapper0 => 0,
            MapperState::Mapper1 { .. } => 1,
            MapperState::Mapper2 { .. } => 2,
            MapperState::Mapper3 { .. } => 3,
            MapperState::Mapper4 { .. } => 4,
            MapperState::Mapper7 { .. } => 7,
            MapperState::Other => 255,
        }
    }
    
    pub fn get_mapper_state(&self) -> MapperState {
        self.mapper.get_state()
    }
    
    pub fn set_mapper_state(&mut self, state: &MapperState) {
        self.mapper.set_state(state);
    }
}
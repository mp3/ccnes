use crate::cartridge::{Mapper, MapperState};

#[derive(Debug)]
pub struct Mapper3 {
    chr_bank: u8,
}

impl Mapper3 {
    pub fn new() -> Self {
        Self {
            chr_bank: 0,
        }
    }
}

impl Mapper for Mapper3 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let index = if prg_rom.len() == 16384 {
                    // 16KB ROM - mirror it
                    (addr & 0x3FFF) as usize
                } else {
                    // 32KB ROM
                    (addr & 0x7FFF) as usize
                };
                prg_rom[index]
            }
            _ => 0,
        }
    }
    
    fn write_prg(&mut self, addr: u16, value: u8) {
        if addr >= 0x8000 {
            self.chr_bank = value & 0x03;
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        if addr < 0x2000 && !chr_rom.is_empty() {
            let offset = addr as usize + (self.chr_bank as usize * 0x2000);
            if offset < chr_rom.len() {
                chr_rom[offset]
            } else {
                0
            }
        } else {
            0
        }
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // No CHR RAM on mapper 3
    }
    
    fn get_state(&self) -> MapperState {
        MapperState::Mapper3 {
            chr_bank: self.chr_bank,
        }
    }
    
    fn set_state(&mut self, state: &MapperState) {
        if let MapperState::Mapper3 { chr_bank } = state {
            self.chr_bank = *chr_bank;
        }
    }
}
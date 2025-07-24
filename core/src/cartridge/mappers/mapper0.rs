use crate::cartridge::Mapper;

#[derive(Debug)]
pub struct Mapper0;

impl Mapper0 {
    pub fn new() -> Self {
        Self
    }
}

impl Mapper for Mapper0 {
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
    
    fn write_prg(&mut self, _addr: u16, _value: u8) {
        // Mapper 0 doesn't support PRG writes
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        if addr < 0x2000 && !chr_rom.is_empty() {
            chr_rom[addr as usize]
        } else {
            0
        }
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // Mapper 0 doesn't support CHR writes for ROM
        // CHR RAM would be handled differently
    }
}
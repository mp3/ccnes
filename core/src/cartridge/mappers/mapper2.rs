use crate::cartridge::Mapper;

#[derive(Debug)]
pub struct Mapper2 {
    prg_bank: u8,
    prg_banks: u8,
}

impl Mapper2 {
    pub fn new(prg_size: usize) -> Self {
        Self {
            prg_bank: 0,
            prg_banks: (prg_size / 0x4000) as u8,
        }
    }
}

impl Mapper for Mapper2 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                // Switchable 16KB bank
                let offset = (addr & 0x3FFF) as usize;
                prg_rom[self.prg_bank as usize * 0x4000 + offset]
            }
            0xC000..=0xFFFF => {
                // Fixed last 16KB bank
                let offset = (addr & 0x3FFF) as usize;
                let last_bank = self.prg_banks - 1;
                prg_rom[last_bank as usize * 0x4000 + offset]
            }
            _ => 0,
        }
    }
    
    fn write_prg(&mut self, addr: u16, value: u8) {
        if addr >= 0x8000 {
            self.prg_bank = value & 0x0F;
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        if addr < 0x2000 && !chr_rom.is_empty() {
            chr_rom[addr as usize]
        } else {
            0
        }
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // CHR RAM if present
    }
}
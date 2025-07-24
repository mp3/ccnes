use crate::cartridge::Mapper;

// Mapper 11: Color Dreams
// Used by some unlicensed games
// Features:
// - 32KB PRG ROM bank switching
// - 8KB CHR ROM bank switching
// - Very simple mapper with single register
#[derive(Debug)]
pub struct Mapper11 {
    prg_bank: usize,
    chr_bank: usize,
    prg_rom_size: usize,
    chr_rom_size: usize,
}

impl Mapper11 {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize) -> Self {
        Self {
            prg_bank: 0,
            chr_bank: 0,
            prg_rom_size,
            chr_rom_size,
        }
    }
}

impl Mapper for Mapper11 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                // 32KB PRG ROM bank
                let offset = (addr - 0x8000) as usize;
                let bank_offset = self.prg_bank * 0x8000;
                prg_rom.get(bank_offset + offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }
    
    fn write_prg(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0xFFFF => {
                // Bits 0-1: PRG bank
                self.prg_bank = (value & 0x03) as usize;
                
                // Bits 4-7: CHR bank
                self.chr_bank = ((value >> 4) & 0x0F) as usize;
                
                // Ensure banks don't exceed ROM sizes
                let max_prg_bank = (self.prg_rom_size / 0x8000).saturating_sub(1);
                self.prg_bank = self.prg_bank.min(max_prg_bank);
                
                let max_chr_bank = (self.chr_rom_size / 0x2000).saturating_sub(1);
                self.chr_bank = self.chr_bank.min(max_chr_bank);
            }
            _ => {}
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        // 8KB CHR ROM bank
        let bank_offset = self.chr_bank * 0x2000;
        let offset = addr as usize;
        chr_rom.get(bank_offset + offset).copied().unwrap_or(0)
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // CHR ROM is not writable in mapper 11
    }
}
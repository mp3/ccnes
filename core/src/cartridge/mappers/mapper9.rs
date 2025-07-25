use crate::cartridge::{Mapper, MapperState};

// Mapper 9: MMC2 (Memory Management Controller 2)
// Used by Mike Tyson's Punch-Out!!
// Features:
// - 8KB PRG ROM bank switching
// - 4KB CHR ROM bank switching with automatic latch
// - Special CHR bank switching triggered by reading tiles $FD/$FE
#[derive(Debug)]
pub struct Mapper9 {
    // ROM sizes
    prg_rom_size: usize,
    chr_rom_size: usize,
    
    // PRG bank (8KB at $A000-$BFFF)
    prg_bank: usize,
    
    // CHR banks and latches
    chr_bank_0: [usize; 2],  // Banks for $0000-$0FFF (FD/FE latch)
    chr_bank_1: [usize; 2],  // Banks for $1000-$1FFF (FD/FE latch)
    latch_0: usize,          // 0 = $FD, 1 = $FE
    latch_1: usize,          // 0 = $FD, 1 = $FE
    
    // PRG RAM
    prg_ram: Vec<u8>,
    
    // Mirroring
    mirroring_mode: u8,
}

impl Mapper9 {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize) -> Self {
        Self {
            prg_rom_size,
            chr_rom_size,
            prg_bank: 0,
            chr_bank_0: [0, 0],
            chr_bank_1: [0, 0],
            latch_0: 0,
            latch_1: 0,
            prg_ram: vec![0; 0x2000], // 8KB PRG RAM
            mirroring_mode: 0,
        }
    }
    
    fn update_chr_latch(&mut self, addr: u16) {
        // Check for special tiles that trigger latch
        match addr {
            0x0FD8..=0x0FDF => {
                // Reading tile $FD in first pattern table
                self.latch_0 = 0;
            }
            0x0FE8..=0x0FEF => {
                // Reading tile $FE in first pattern table
                self.latch_0 = 1;
            }
            0x1FD8..=0x1FDF => {
                // Reading tile $FD in second pattern table
                self.latch_1 = 0;
            }
            0x1FE8..=0x1FEF => {
                // Reading tile $FE in second pattern table
                self.latch_1 = 1;
            }
            _ => {}
        }
    }
}

impl Mapper for Mapper9 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                // PRG RAM
                self.prg_ram[(addr - 0x6000) as usize]
            }
            0x8000..=0x9FFF => {
                // First 8KB PRG ROM bank (fixed to first bank)
                let offset = (addr - 0x8000) as usize;
                prg_rom.get(offset).copied().unwrap_or(0)
            }
            0xA000..=0xBFFF => {
                // 8KB switchable PRG ROM bank
                let offset = (addr - 0xA000) as usize;
                let bank_offset = self.prg_bank * 0x2000;
                prg_rom.get(bank_offset + offset).copied().unwrap_or(0)
            }
            0xC000..=0xFFFF => {
                // Last 16KB PRG ROM bank (fixed to last 16KB)
                let offset = (addr - 0xC000) as usize;
                let bank_offset = self.prg_rom_size.saturating_sub(0x4000);
                prg_rom.get(bank_offset + offset).copied().unwrap_or(0)
            }
            _ => 0,
        }
    }
    
    fn write_prg(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000..=0x7FFF => {
                // PRG RAM
                self.prg_ram[(addr - 0x6000) as usize] = value;
            }
            0xA000..=0xAFFF => {
                // PRG ROM bank select
                self.prg_bank = (value & 0x0F) as usize;
                // Ensure bank doesn't exceed ROM size
                let max_bank = (self.prg_rom_size / 0x2000).saturating_sub(1);
                self.prg_bank = self.prg_bank.min(max_bank);
            }
            0xB000..=0xBFFF => {
                // CHR ROM bank select for $0000-$0FFF (FD latch)
                self.chr_bank_0[0] = (value & 0x1F) as usize;
            }
            0xC000..=0xCFFF => {
                // CHR ROM bank select for $0000-$0FFF (FE latch)
                self.chr_bank_0[1] = (value & 0x1F) as usize;
            }
            0xD000..=0xDFFF => {
                // CHR ROM bank select for $1000-$1FFF (FD latch)
                self.chr_bank_1[0] = (value & 0x1F) as usize;
            }
            0xE000..=0xEFFF => {
                // CHR ROM bank select for $1000-$1FFF (FE latch)
                self.chr_bank_1[1] = (value & 0x1F) as usize;
            }
            0xF000..=0xFFFF => {
                // Mirroring control
                self.mirroring_mode = value & 1;
            }
            _ => {}
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        // Note: Can't update latch on read in this trait design
        // The actual hardware updates the latch when specific tiles are read
        
        let (bank, offset) = match addr {
            0x0000..=0x0FFF => {
                // Use latch_0 to select bank
                let bank = self.chr_bank_0[self.latch_0];
                (bank, addr as usize)
            }
            0x1000..=0x1FFF => {
                // Use latch_1 to select bank
                let bank = self.chr_bank_1[self.latch_1];
                (bank, (addr - 0x1000) as usize)
            }
            _ => return 0,
        };
        
        let bank_offset = bank * 0x1000; // 4KB banks
        chr_rom.get(bank_offset + offset).copied().unwrap_or(0)
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // CHR ROM is not writable in mapper 9
    }
    
    fn get_state(&self) -> MapperState {
        MapperState::Other
    }
    
    fn set_state(&mut self, _state: &MapperState) {
        // Mapper 9 state restoration not implemented yet
    }
}
use crate::cartridge::{Mapper, MapperState};

// Mapper 5: MMC5 (Memory Management Controller 5)
// One of the most complex mappers, used by games like Castlevania III
// This is a simplified implementation of core features
#[derive(Debug)]
pub struct Mapper5 {
    // ROM sizes
    prg_rom_size: usize,
    chr_rom_size: usize,
    
    // PRG banks (8KB each)
    prg_banks: [usize; 4],
    prg_mode: u8,
    
    // CHR banks (1KB each)  
    chr_banks: [usize; 12],
    chr_mode: u8,
    
    // RAM banks
    prg_ram_banks: [usize; 2],
    prg_ram: Vec<u8>,
    
    // Fill mode tile and attribute
    fill_tile: u8,
    fill_attr: u8,
    
    // Mirroring
    mirroring_mode: u8,
    
    // IRQ
    irq_enabled: bool,
    irq_pending: bool,
    irq_scanline: u8,
    irq_compare: u8,
    in_frame: bool,
    
    // Extended RAM (ExRAM)
    exram: Vec<u8>,
    exram_mode: u8,
}

impl Mapper5 {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize) -> Self {
        Self {
            prg_rom_size,
            chr_rom_size,
            prg_banks: [0; 4],
            prg_mode: 3, // Default to mode 3
            chr_banks: [0; 12],
            chr_mode: 0,
            prg_ram_banks: [0; 2],
            prg_ram: vec![0; 0x10000], // 64KB PRG RAM
            fill_tile: 0,
            fill_attr: 0,
            mirroring_mode: 0,
            irq_enabled: false,
            irq_pending: false,
            irq_scanline: 0,
            irq_compare: 0,
            in_frame: false,
            exram: vec![0; 0x400], // 1KB Extended RAM
            exram_mode: 0,
        }
    }
    
    fn get_prg_bank(&self, addr: u16) -> (usize, usize) {
        let bank_index = match self.prg_mode {
            0 => {
                // Mode 0: 32KB banks
                match addr {
                    0x8000..=0xFFFF => self.prg_banks[3] >> 2,
                    _ => 0,
                }
            }
            1 => {
                // Mode 1: 16KB banks
                match addr {
                    0x8000..=0xBFFF => self.prg_banks[1] >> 1,
                    0xC000..=0xFFFF => self.prg_banks[3] >> 1,
                    _ => 0,
                }
            }
            2 => {
                // Mode 2: 16KB + 8KB banks
                match addr {
                    0x8000..=0xBFFF => self.prg_banks[1] >> 1,
                    0xC000..=0xDFFF => self.prg_banks[2],
                    0xE000..=0xFFFF => self.prg_banks[3],
                    _ => 0,
                }
            }
            3 => {
                // Mode 3: 8KB banks
                match addr {
                    0x8000..=0x9FFF => self.prg_banks[0],
                    0xA000..=0xBFFF => self.prg_banks[1],
                    0xC000..=0xDFFF => self.prg_banks[2],
                    0xE000..=0xFFFF => self.prg_banks[3],
                    _ => 0,
                }
            }
            _ => 0,
        };
        
        let offset = match self.prg_mode {
            0 => (addr & 0x7FFF) as usize,
            1 => (addr & 0x3FFF) as usize,
            _ => (addr & 0x1FFF) as usize,
        };
        
        (bank_index, offset)
    }
}

impl Mapper for Mapper5 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x5C00..=0x5FFF => {
                // Extended RAM
                if self.exram_mode < 2 {
                    self.exram[(addr - 0x5C00) as usize]
                } else {
                    0
                }
            }
            0x6000..=0x7FFF => {
                // PRG RAM bank 0
                let bank_offset = self.prg_ram_banks[0] * 0x2000;
                let offset = (addr - 0x6000) as usize;
                self.prg_ram.get(bank_offset + offset).copied().unwrap_or(0)
            }
            0x8000..=0xFFFF => {
                let (bank, offset) = self.get_prg_bank(addr);
                let bank_offset = bank * 0x2000;
                
                // Check if this is RAM or ROM
                if bank >= 0x80 {
                    // RAM bank
                    let ram_bank = (bank - 0x80) & 0x07;
                    let ram_offset = ram_bank * 0x2000 + offset;
                    self.prg_ram.get(ram_offset).copied().unwrap_or(0)
                } else {
                    // ROM bank
                    prg_rom.get(bank_offset + offset).copied().unwrap_or(0)
                }
            }
            _ => 0,
        }
    }
    
    fn write_prg(&mut self, addr: u16, value: u8) {
        match addr {
            0x5000..=0x5015 => {
                // Audio registers (not implemented)
            }
            0x5100 => {
                // PRG mode
                self.prg_mode = value & 0x03;
            }
            0x5101 => {
                // CHR mode
                self.chr_mode = value & 0x03;
            }
            0x5102 => {
                // PRG RAM protect 1 (not implemented)
            }
            0x5103 => {
                // PRG RAM protect 2 (not implemented)
            }
            0x5104 => {
                // Extended RAM mode
                self.exram_mode = value & 0x03;
            }
            0x5105 => {
                // Nametable mapping (simplified)
                self.mirroring_mode = value;
            }
            0x5106 => {
                // Fill mode tile
                self.fill_tile = value;
            }
            0x5107 => {
                // Fill mode attribute
                self.fill_attr = value & 0x03;
            }
            0x5113 => {
                // PRG RAM bank
                self.prg_ram_banks[0] = (value & 0x07) as usize;
            }
            0x5114..=0x5117 => {
                // PRG ROM banks
                let bank_index = (addr - 0x5114) as usize;
                self.prg_banks[bank_index] = value as usize;
            }
            0x5120..=0x512B => {
                // CHR banks
                let bank_index = (addr - 0x5120) as usize;
                self.chr_banks[bank_index] = value as usize | ((value as usize) << 8);
            }
            0x5200 => {
                // Vertical split mode (not implemented)
            }
            0x5203 => {
                // IRQ scanline compare value
                self.irq_compare = value;
            }
            0x5204 => {
                // IRQ enable
                self.irq_enabled = (value & 0x80) != 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
            }
            0x5C00..=0x5FFF => {
                // Extended RAM
                if self.exram_mode == 0 || self.exram_mode == 1 {
                    self.exram[(addr - 0x5C00) as usize] = value;
                }
            }
            0x6000..=0x7FFF => {
                // PRG RAM
                let bank_offset = self.prg_ram_banks[0] * 0x2000;
                let offset = (addr - 0x6000) as usize;
                if let Some(byte) = self.prg_ram.get_mut(bank_offset + offset) {
                    *byte = value;
                }
            }
            _ => {}
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        let bank_index = match self.chr_mode {
            0 => {
                // 8KB mode
                (addr >> 13) as usize
            }
            1 => {
                // 4KB mode
                (addr >> 12) as usize
            }
            2 => {
                // 2KB mode
                (addr >> 11) as usize
            }
            3 => {
                // 1KB mode
                (addr >> 10) as usize
            }
            _ => 0,
        };
        
        let bank = self.chr_banks[bank_index];
        let offset = addr as usize & match self.chr_mode {
            0 => 0x1FFF,
            1 => 0x0FFF,
            2 => 0x07FF,
            3 => 0x03FF,
            _ => 0,
        };
        
        let bank_offset = bank * 0x400; // 1KB units
        chr_rom.get(bank_offset + offset).copied().unwrap_or(0)
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // CHR ROM is not writable in mapper 5
    }
    
    fn get_state(&self) -> MapperState {
        // Mapper 5 is complex - return Other for now
        MapperState::Other
    }
    
    fn set_state(&mut self, _state: &MapperState) {
        // Mapper 5 state restoration not implemented yet
    }
}
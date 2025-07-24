use crate::cartridge::Mapper;

// Mapper 4: MMC3 (Memory Management Controller 3)
// Used by many popular games like Super Mario Bros. 3, Mega Man 3-6, etc.
#[derive(Debug)]
pub struct Mapper4 {
    // Bank registers
    bank_select: u8,
    bank_data: [u8; 8],
    
    // PRG banks (8KB each)
    prg_banks: [usize; 4],
    
    // CHR banks (1KB each)
    chr_banks: [usize; 8],
    
    // Mirroring
    mirroring_mode: u8,
    
    // IRQ
    irq_enabled: bool,
    irq_counter: u8,
    irq_latch: u8,
    irq_reload: bool,
    irq_pending: bool,
    
    // Scanline counter
    last_a12: bool,
    a12_filter: u8,
    
    // RAM
    prg_ram: Vec<u8>,
    chr_ram: Vec<u8>,
    prg_rom_size: usize,
    chr_rom_size: usize,
}

impl Mapper4 {
    pub fn new(prg_rom_size: usize, chr_rom_size: usize) -> Self {
        let prg_banks = [
            0,
            0x2000,
            prg_rom_size.saturating_sub(0x4000),
            prg_rom_size.saturating_sub(0x2000),
        ];
        
        let chr_banks = [0, 0, 0, 0, 0, 0, 0, 0];
        
        Self {
            bank_select: 0,
            bank_data: [0; 8],
            prg_banks,
            chr_banks,
            mirroring_mode: 0,
            irq_enabled: false,
            irq_counter: 0,
            irq_latch: 0,
            irq_reload: false,
            irq_pending: false,
            last_a12: false,
            a12_filter: 0,
            prg_ram: vec![0; 0x2000], // 8KB PRG RAM
            chr_ram: if chr_rom_size == 0 { vec![0; 0x2000] } else { vec![] },
            prg_rom_size,
            chr_rom_size,
        }
    }
    
    fn update_banks(&mut self) {
        // PRG mode
        let prg_mode = (self.bank_select >> 6) & 1;
        
        if prg_mode == 0 {
            // Mode 0: $8000-$9FFF swappable, $C000-$DFFF fixed to second-last bank
            self.prg_banks[0] = (self.bank_data[6] as usize & 0x3F) * 0x2000;
            self.prg_banks[1] = (self.bank_data[7] as usize & 0x3F) * 0x2000;
            self.prg_banks[2] = self.prg_rom_size - 0x4000;
        } else {
            // Mode 1: $C000-$DFFF swappable, $8000-$9FFF fixed to second-last bank
            self.prg_banks[0] = self.prg_rom_size - 0x4000;
            self.prg_banks[1] = (self.bank_data[7] as usize & 0x3F) * 0x2000;
            self.prg_banks[2] = (self.bank_data[6] as usize & 0x3F) * 0x2000;
        }
        
        // CHR mode
        let chr_mode = (self.bank_select >> 7) & 1;
        
        if chr_mode == 0 {
            // Mode 0: 2KB banks at $0000-$0FFF, 1KB banks at $1000-$1FFF
            self.chr_banks[0] = (self.bank_data[0] as usize & 0xFE) * 0x400;
            self.chr_banks[1] = (self.bank_data[0] as usize | 0x01) * 0x400;
            self.chr_banks[2] = (self.bank_data[1] as usize & 0xFE) * 0x400;
            self.chr_banks[3] = (self.bank_data[1] as usize | 0x01) * 0x400;
            self.chr_banks[4] = (self.bank_data[2] as usize) * 0x400;
            self.chr_banks[5] = (self.bank_data[3] as usize) * 0x400;
            self.chr_banks[6] = (self.bank_data[4] as usize) * 0x400;
            self.chr_banks[7] = (self.bank_data[5] as usize) * 0x400;
        } else {
            // Mode 1: 1KB banks at $0000-$0FFF, 2KB banks at $1000-$1FFF
            self.chr_banks[0] = (self.bank_data[2] as usize) * 0x400;
            self.chr_banks[1] = (self.bank_data[3] as usize) * 0x400;
            self.chr_banks[2] = (self.bank_data[4] as usize) * 0x400;
            self.chr_banks[3] = (self.bank_data[5] as usize) * 0x400;
            self.chr_banks[4] = (self.bank_data[0] as usize & 0xFE) * 0x400;
            self.chr_banks[5] = (self.bank_data[0] as usize | 0x01) * 0x400;
            self.chr_banks[6] = (self.bank_data[1] as usize & 0xFE) * 0x400;
            self.chr_banks[7] = (self.bank_data[1] as usize | 0x01) * 0x400;
        }
        
        // Ensure banks don't exceed ROM size
        for bank in &mut self.prg_banks {
            *bank = (*bank).min(self.prg_rom_size.saturating_sub(0x2000));
        }
        
        if self.chr_rom_size > 0 {
            for bank in &mut self.chr_banks {
                *bank = (*bank).min(self.chr_rom_size.saturating_sub(0x400));
            }
        }
    }
    
    fn clock_scanline(&mut self, addr: u16) {
        // Detect PPU A12 rising edge (used for scanline counting)
        let a12 = (addr & 0x1000) != 0;
        
        if !self.last_a12 && a12 {
            // Filter to prevent false triggers
            if self.a12_filter == 0 {
                self.a12_filter = 15;
                
                if self.irq_counter == 0 || self.irq_reload {
                    self.irq_counter = self.irq_latch;
                    self.irq_reload = false;
                } else {
                    self.irq_counter = self.irq_counter.wrapping_sub(1);
                }
                
                if self.irq_counter == 0 && self.irq_enabled {
                    self.irq_pending = true;
                }
            }
        }
        
        if self.a12_filter > 0 {
            self.a12_filter -= 1;
        }
        
        self.last_a12 = a12;
    }
}

impl Mapper for Mapper4 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                // PRG RAM
                self.prg_ram[(addr - 0x6000) as usize]
            }
            0x8000..=0x9FFF => {
                let offset = (addr - 0x8000) as usize;
                prg_rom.get(self.prg_banks[0] + offset).copied().unwrap_or(0)
            }
            0xA000..=0xBFFF => {
                let offset = (addr - 0xA000) as usize;
                prg_rom.get(self.prg_banks[1] + offset).copied().unwrap_or(0)
            }
            0xC000..=0xDFFF => {
                let offset = (addr - 0xC000) as usize;
                prg_rom.get(self.prg_banks[2] + offset).copied().unwrap_or(0)
            }
            0xE000..=0xFFFF => {
                let offset = (addr - 0xE000) as usize;
                prg_rom.get(self.prg_banks[3] + offset).copied().unwrap_or(0)
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
            0x8000..=0x9FFF => {
                if addr & 1 == 0 {
                    // Bank select ($8000, even)
                    self.bank_select = value;
                } else {
                    // Bank data ($8001, odd)
                    let bank_num = (self.bank_select & 0x07) as usize;
                    self.bank_data[bank_num] = value;
                    self.update_banks();
                }
            }
            0xA000..=0xBFFF => {
                if addr & 1 == 0 {
                    // Mirroring ($A000, even)
                    self.mirroring_mode = value & 1;
                } else {
                    // PRG RAM protect ($A001, odd) - not implemented
                }
            }
            0xC000..=0xDFFF => {
                if addr & 1 == 0 {
                    // IRQ latch ($C000, even)
                    self.irq_latch = value;
                } else {
                    // IRQ reload ($C001, odd)
                    self.irq_reload = true;
                    self.irq_counter = 0;
                }
            }
            0xE000..=0xFFFF => {
                if addr & 1 == 0 {
                    // IRQ disable ($E000, even)
                    self.irq_enabled = false;
                    self.irq_pending = false;
                } else {
                    // IRQ enable ($E001, odd)
                    self.irq_enabled = true;
                }
            }
            _ => {}
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        // Note: Can't clock scanline on reads in this trait design
        
        if chr_rom.is_empty() {
            // CHR RAM
            self.chr_ram.get(addr as usize).copied().unwrap_or(0)
        } else {
            let bank = (addr / 0x400) as usize;
            let offset = (addr % 0x400) as usize;
            chr_rom.get(self.chr_banks[bank] + offset).copied().unwrap_or(0)
        }
    }
    
    fn write_chr(&mut self, addr: u16, value: u8) {
        // Note: Can't clock scanline on writes in this trait design
        
        if self.chr_rom_size == 0 && addr < 0x2000 {
            // CHR RAM
            self.chr_ram[addr as usize] = value;
        }
    }
}
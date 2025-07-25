use crate::cartridge::{Mapper, MapperState};

#[derive(Debug)]
pub struct Mapper1 {
    shift_register: u8,
    shift_count: u8,
    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
}

impl Mapper1 {
    pub fn new() -> Self {
        Self {
            shift_register: 0x10,
            shift_count: 0,
            control: 0x0C,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }
}

impl Mapper for Mapper1 {
    fn read_prg(&self, addr: u16, prg_rom: &[u8]) -> u8 {
        match addr {
            0x8000..=0xBFFF => {
                let bank = if self.control & 0x08 != 0 {
                    // 16KB mode - use prg_bank
                    self.prg_bank as usize
                } else {
                    // 32KB mode - use lower bank
                    0
                };
                let offset = (addr & 0x3FFF) as usize;
                prg_rom[bank * 0x4000 + offset]
            }
            0xC000..=0xFFFF => {
                let bank = if self.control & 0x08 != 0 {
                    // 16KB mode - fixed to last bank or switchable
                    if self.control & 0x04 != 0 {
                        prg_rom.len() / 0x4000 - 1
                    } else {
                        self.prg_bank as usize
                    }
                } else {
                    // 32KB mode - use upper bank
                    1
                };
                let offset = (addr & 0x3FFF) as usize;
                prg_rom[bank * 0x4000 + offset]
            }
            _ => 0,
        }
    }
    
    fn write_prg(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 {
            return;
        }
        
        if value & 0x80 != 0 {
            // Reset shift register
            self.shift_register = 0x10;
            self.shift_count = 0;
            self.control |= 0x0C;
        } else {
            // Shift in bit
            let complete = self.shift_register & 0x01 != 0;
            self.shift_register >>= 1;
            self.shift_register |= (value & 0x01) << 4;
            self.shift_count += 1;
            
            if complete {
                // Write to internal register
                match (addr >> 13) & 0x03 {
                    0 => self.control = self.shift_register,
                    1 => self.chr_bank0 = self.shift_register,
                    2 => self.chr_bank1 = self.shift_register,
                    3 => self.prg_bank = self.shift_register & 0x0F,
                    _ => unreachable!(),
                }
                
                // Reset shift register
                self.shift_register = 0x10;
                self.shift_count = 0;
            }
        }
    }
    
    fn read_chr(&self, addr: u16, chr_rom: &[u8]) -> u8 {
        if addr >= 0x2000 || chr_rom.is_empty() {
            return 0;
        }
        
        let bank = match addr {
            0x0000..=0x0FFF => {
                if self.control & 0x10 != 0 {
                    // 4KB mode
                    self.chr_bank0 as usize
                } else {
                    // 8KB mode
                    (self.chr_bank0 & 0xFE) as usize
                }
            }
            0x1000..=0x1FFF => {
                if self.control & 0x10 != 0 {
                    // 4KB mode
                    self.chr_bank1 as usize
                } else {
                    // 8KB mode
                    ((self.chr_bank0 & 0xFE) | 1) as usize
                }
            }
            _ => return 0,
        };
        
        let offset = (addr & 0x0FFF) as usize;
        chr_rom[bank * 0x1000 + offset]
    }
    
    fn write_chr(&mut self, _addr: u16, _value: u8) {
        // CHR writes would go to CHR RAM if present
    }
    
    fn get_state(&self) -> MapperState {
        MapperState::Mapper1 {
            shift_register: self.shift_register,
            shift_count: self.shift_count,
            prg_bank_mode: (self.control >> 2) & 0x03,
            chr_bank_mode: (self.control >> 4) & 0x01,
            prg_bank: self.prg_bank,
            chr_bank0: self.chr_bank0,
            chr_bank1: self.chr_bank1,
        }
    }
    
    fn set_state(&mut self, state: &MapperState) {
        if let MapperState::Mapper1 {
            shift_register,
            shift_count,
            prg_bank_mode,
            chr_bank_mode,
            prg_bank,
            chr_bank0,
            chr_bank1,
        } = state {
            self.shift_register = *shift_register;
            self.shift_count = *shift_count;
            self.control = (*prg_bank_mode << 2) | (*chr_bank_mode << 4);
            self.prg_bank = *prg_bank;
            self.chr_bank0 = *chr_bank0;
            self.chr_bank1 = *chr_bank1;
        }
    }
}
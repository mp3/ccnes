use crate::cartridge::Mapper;

// Mapper 7: AxROM
// Used by games like Battletoads, Wizards & Warriors, etc.
// Features:
// - 32KB PRG ROM bank switching
// - Single screen mirroring
// - No CHR ROM (uses CHR RAM)
#[derive(Debug)]
pub struct Mapper7 {
    chr_ram: Vec<u8>,
    prg_bank: usize,
    mirroring_mode: u8,
    prg_rom_size: usize,
}

impl Mapper7 {
    pub fn new(prg_rom_size: usize) -> Self {
        Self {
            chr_ram: vec![0; 0x2000], // 8KB CHR RAM
            prg_bank: 0,
            mirroring_mode: 0,
            prg_rom_size,
        }
    }
}

impl Mapper for Mapper7 {
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
                // Bus conflicts: The written data must match the data on the bus
                let current_data = self.read_prg(addr, &[]);
                let actual_value = if current_data != 0 {
                    // Bus conflict occurred - data is ANDed together
                    value & current_data
                } else {
                    value
                };
                
                // Bank select (bits 0-2) and mirroring (bit 4)
                self.prg_bank = (actual_value & 0x07) as usize;
                self.mirroring_mode = (actual_value >> 4) & 1;
                
                // Ensure bank doesn't exceed ROM size
                let max_bank = (self.prg_rom_size / 0x8000).saturating_sub(1);
                self.prg_bank = self.prg_bank.min(max_bank);
            }
            _ => {}
        }
    }
    
    fn read_chr(&self, addr: u16, _chr_rom: &[u8]) -> u8 {
        // Always uses CHR RAM
        self.chr_ram.get(addr as usize & 0x1FFF).copied().unwrap_or(0)
    }
    
    fn write_chr(&mut self, addr: u16, value: u8) {
        // CHR RAM
        if addr < 0x2000 {
            self.chr_ram[addr as usize] = value;
        }
    }
}
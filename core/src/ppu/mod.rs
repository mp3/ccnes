#[derive(Debug, Clone)]
pub struct Ppu {
    // PPU registers
    ctrl: u8,      // $2000 PPUCTRL
    mask: u8,      // $2001 PPUMASK
    status: u8,    // $2002 PPUSTATUS
    oam_addr: u8,  // $2003 OAMADDR
    scroll_x: u8,
    scroll_y: u8,
    addr: u16,
    
    // Internal memory
    vram: [u8; 0x800],     // 2KB VRAM (name tables)
    palette: [u8; 32],     // Palette RAM
    oam: [u8; 256],        // Object Attribute Memory
    
    // Rendering state
    scanline: u16,
    cycle: u16,
    frame: u64,
    
    // Temporary variables
    write_toggle: bool,
    buffer: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            scroll_x: 0,
            scroll_y: 0,
            addr: 0,
            vram: [0; 0x800],
            palette: [0; 32],
            oam: [0; 256],
            scanline: 0,
            cycle: 0,
            frame: 0,
            write_toggle: false,
            buffer: 0,
        }
    }
    
    pub fn read_register(&self, reg: u8) -> u8 {
        match reg {
            2 => {
                // PPUSTATUS
                // Reading clears vblank flag and write toggle
                let value = self.status;
                // Note: In a real implementation, we'd clear bit 7 and reset write_toggle
                value
            }
            4 => {
                // OAMDATA
                self.oam[self.oam_addr as usize]
            }
            7 => {
                // PPUDATA
                // This would involve proper buffering in a real implementation
                self.buffer
            }
            _ => 0,
        }
    }
    
    pub fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0 => self.ctrl = value,      // PPUCTRL
            1 => self.mask = value,      // PPUMASK
            3 => self.oam_addr = value,  // OAMADDR
            4 => {
                // OAMDATA
                self.oam[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                // PPUSCROLL
                if !self.write_toggle {
                    self.scroll_x = value;
                } else {
                    self.scroll_y = value;
                }
                self.write_toggle = !self.write_toggle;
            }
            6 => {
                // PPUADDR
                if !self.write_toggle {
                    self.addr = (self.addr & 0x00FF) | ((value as u16) << 8);
                } else {
                    self.addr = (self.addr & 0xFF00) | (value as u16);
                }
                self.write_toggle = !self.write_toggle;
            }
            7 => {
                // PPUDATA
                self.write_vram(self.addr, value);
                self.addr = self.addr.wrapping_add(self.addr_increment());
            }
            _ => {}
        }
    }
    
    fn addr_increment(&self) -> u16 {
        if self.ctrl & 0x04 != 0 {
            32
        } else {
            1
        }
    }
    
    fn write_vram(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables - write to cartridge CHR
            }
            0x2000..=0x2FFF => {
                // Name tables
                self.vram[(addr & 0x7FF) as usize] = value;
            }
            0x3000..=0x3EFF => {
                // Mirrors of 0x2000-0x2EFF
                self.vram[(addr & 0x7FF) as usize] = value;
            }
            0x3F00..=0x3FFF => {
                // Palette
                let palette_addr = (addr & 0x1F) as usize;
                self.palette[palette_addr] = value;
            }
            _ => {}
        }
    }
    
    pub fn step(&mut self) -> bool {
        self.cycle += 1;
        
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame += 1;
            }
        }
        
        // Return true if we need to trigger NMI
        self.scanline == 241 && self.cycle == 1 && (self.ctrl & 0x80) != 0
    }
}
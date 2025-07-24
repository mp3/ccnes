use crate::cartridge::Cartridge;

mod palette;
use palette::NES_PALETTE;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

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
    secondary_oam: [u8; 32], // Secondary OAM for sprite evaluation
    
    // Rendering state
    scanline: i32,
    cycle: i32,
    frame: u64,
    
    // Temporary variables
    write_toggle: bool,
    buffer: u8,
    
    // Internal latches and shift registers
    v: u16,        // Current VRAM address
    t: u16,        // Temporary VRAM address
    x: u8,         // Fine X scroll
    w: bool,       // Write toggle
    
    // Background shift registers
    bg_shift_pattern_lo: u16,
    bg_shift_pattern_hi: u16,
    bg_shift_attrib_lo: u16,
    bg_shift_attrib_hi: u16,
    
    // Sprite rendering
    sprite_count: u8,
    sprite_patterns: [u8; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indexes: [u8; 8],
    
    // Frame buffer
    pub framebuffer: Vec<u32>,
    
    // NMI output
    nmi_output: bool,
    nmi_occurred: bool,
    
    // Odd frame flag
    odd_frame: bool,
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
            secondary_oam: [0; 32],
            scanline: 0,
            cycle: 0,
            frame: 0,
            write_toggle: false,
            buffer: 0,
            v: 0,
            t: 0,
            x: 0,
            w: false,
            bg_shift_pattern_lo: 0,
            bg_shift_pattern_hi: 0,
            bg_shift_attrib_lo: 0,
            bg_shift_attrib_hi: 0,
            sprite_count: 0,
            sprite_patterns: [0; 8],
            sprite_positions: [0; 8],
            sprite_priorities: [0; 8],
            sprite_indexes: [0; 8],
            framebuffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            nmi_output: false,
            nmi_occurred: false,
            odd_frame: false,
        }
    }
    
    pub fn read_register(&mut self, reg: u8) -> u8 {
        match reg {
            2 => {
                // PPUSTATUS
                let value = (self.status & 0xE0) | (self.buffer & 0x1F);
                self.status &= !0x80;  // Clear vblank flag
                self.w = false;         // Reset write toggle
                value
            }
            4 => {
                // OAMDATA
                self.oam[self.oam_addr as usize]
            }
            7 => {
                // PPUDATA
                let addr = self.v & 0x3FFF;
                let mut value = self.buffer;
                
                if addr < 0x3F00 {
                    self.buffer = self.read_byte(addr);
                } else {
                    self.buffer = self.read_byte(addr - 0x1000);
                    value = self.read_byte(addr);
                }
                
                self.v = self.v.wrapping_add(self.addr_increment());
                value
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
                if !self.w {
                    self.t = (self.t & 0xFFE0) | ((value as u16) >> 3);
                    self.x = value & 0x07;
                } else {
                    self.t = (self.t & 0x8FFF) | (((value as u16) & 0x07) << 12);
                    self.t = (self.t & 0xFC1F) | (((value as u16) & 0xF8) << 2);
                }
                self.w = !self.w;
            }
            6 => {
                // PPUADDR
                if !self.w {
                    self.t = (self.t & 0x80FF) | (((value as u16) & 0x3F) << 8);
                } else {
                    self.t = (self.t & 0xFF00) | (value as u16);
                    self.v = self.t;
                }
                self.w = !self.w;
            }
            7 => {
                // PPUDATA
                self.write_byte(self.v & 0x3FFF, value);
                self.v = self.v.wrapping_add(self.addr_increment());
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
    
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables - read from cartridge CHR
                0  // Placeholder - should read from cartridge
            }
            0x2000..=0x3EFF => {
                // Name tables and mirrors
                self.vram[self.mirror_address(addr) as usize]
            }
            0x3F00..=0x3FFF => {
                // Palette
                let mut palette_addr = (addr & 0x1F) as usize;
                if palette_addr >= 0x10 && palette_addr % 4 == 0 {
                    palette_addr &= 0x0F;
                }
                self.palette[palette_addr] & if self.mask & 0x01 != 0 { 0x30 } else { 0x3F }
            }
            _ => 0,
        }
    }
    
    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables - write to cartridge CHR
                // Placeholder - should write to cartridge
            }
            0x2000..=0x3EFF => {
                // Name tables
                let mirrored = self.mirror_address(addr);
                self.vram[mirrored as usize] = value;
            }
            0x3F00..=0x3FFF => {
                // Palette
                let mut palette_addr = (addr & 0x1F) as usize;
                if palette_addr >= 0x10 && palette_addr % 4 == 0 {
                    palette_addr &= 0x0F;
                }
                self.palette[palette_addr] = value;
            }
            _ => {}
        }
    }
    
    fn mirror_address(&self, addr: u16) -> u16 {
        // Simple horizontal mirroring for now
        // TODO: Get actual mirroring mode from cartridge
        let addr = (addr - 0x2000) % 0x1000;
        match addr / 0x400 {
            0 => addr,
            1 => addr - 0x400,
            2 => addr - 0x400,
            3 => addr - 0x800,
            _ => unreachable!(),
        }
    }
    
    pub fn step(&mut self, _cartridge: &Cartridge) -> bool {
        // Visible scanlines (0-239)
        if self.scanline >= 0 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }
            
            if self.cycle >= 1 && self.cycle <= 256 {
                self.render_pixel();
            }
            
            if self.cycle == 256 {
                self.increment_y();
            }
            
            if self.cycle == 257 {
                self.load_background_shifters();
                if self.mask & 0x08 != 0 {
                    self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
                }
            }
            
            if self.cycle == 338 || self.cycle == 340 {
                self.bg_shift_pattern_lo = self.bg_shift_pattern_lo << 8;
                self.bg_shift_pattern_hi = self.bg_shift_pattern_hi << 8;
            }
            
            if self.scanline == 239 && self.cycle == 340 {
                // Frame complete
            }
        }
        
        // Post-render scanline (240)
        if self.scanline == 240 {
            // Nothing happens here
        }
        
        // Vertical blanking scanlines (241-260)
        if self.scanline >= 241 && self.scanline <= 260 {
            if self.scanline == 241 && self.cycle == 1 {
                self.status |= 0x80;  // Set vblank flag
                self.nmi_occurred = true;
                if self.nmi_output {
                    // NMI will be triggered
                }
            }
        }
        
        // Pre-render scanline (261 / -1)
        if self.scanline == 261 {
            if self.cycle == 1 {
                self.status &= !0x80;  // Clear vblank flag
                self.status &= !0x40;  // Clear sprite 0 hit
                self.status &= !0x20;  // Clear sprite overflow
            }
            
            if self.cycle >= 280 && self.cycle <= 304 {
                if self.mask & 0x18 != 0 {
                    self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
                }
            }
            
            if self.cycle == 339 && self.odd_frame && self.mask & 0x18 != 0 {
                self.cycle = 340;
            }
        }
        
        let nmi = self.nmi_occurred && self.nmi_output;
        
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            
            if self.scanline > 261 {
                self.scanline = -1;
                self.frame += 1;
                self.odd_frame = !self.odd_frame;
            }
        }
        
        nmi
    }
    
    fn render_pixel(&mut self) {
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;
        
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            let pixel_offset = y * SCREEN_WIDTH + x;
            
            // For now, use palette entry 0 (universal background color)
            // TODO: Implement actual background and sprite rendering
            let palette_index = self.palette[0] & 0x3F;
            let color = NES_PALETTE[palette_index as usize];
            
            self.framebuffer[pixel_offset] = color;
        }
    }
    
    fn increment_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400;
        } else {
            self.v += 1;
        }
    }
    
    fn increment_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000;
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }
    
    fn load_background_shifters(&mut self) {
        self.bg_shift_pattern_lo = (self.bg_shift_pattern_lo & 0xFF00) | 0x00FF;
        self.bg_shift_pattern_hi = (self.bg_shift_pattern_hi & 0xFF00) | 0x00FF;
    }
    
    pub fn get_nmi_output(&self) -> bool {
        self.nmi_output
    }
    
    pub fn set_nmi_output(&mut self, value: bool) {
        let prev = self.nmi_output;
        self.nmi_output = value;
        if value && !prev && self.nmi_occurred {
            // Rising edge of nmi_output when nmi_occurred is true
        }
    }
}
use crate::cartridge::Cartridge;

mod palette;
use palette::NES_PALETTE;
pub mod optimized;

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
    pub palette: [u8; 32],     // Palette RAM
    pub oam: [u8; 256],        // Object Attribute Memory
    secondary_oam: [u8; 32], // Secondary OAM for sprite evaluation
    
    // Rendering state
    scanline: i32,
    cycle: i32,
    frame: u64,
    
    // Temporary variables
    write_toggle: bool,
    buffer: u8,
    
    // PPU open bus
    open_bus: u8,
    
    // Suppress VBlank flag
    suppress_vbl: bool,
    
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
    
    // Background latches
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    
    // Sprite rendering
    sprite_count: u8,
    sprite_patterns_lo: [u8; 8],
    sprite_patterns_hi: [u8; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indexes: [u8; 8],
    sprite_attributes: [u8; 8],
    
    // Frame buffer
    pub framebuffer: Vec<u32>,
    
    // NMI output
    nmi_output: bool,
    nmi_occurred: bool,
    
    // Odd frame flag
    odd_frame: bool,
    
    // Optimized rendering tables
    rendering_tables: optimized::RenderingTables,
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
            open_bus: 0,
            suppress_vbl: false,
            v: 0,
            t: 0,
            x: 0,
            w: false,
            bg_shift_pattern_lo: 0,
            bg_shift_pattern_hi: 0,
            bg_shift_attrib_lo: 0,
            bg_shift_attrib_hi: 0,
            bg_next_tile_id: 0,
            bg_next_tile_attrib: 0,
            bg_next_tile_lsb: 0,
            bg_next_tile_msb: 0,
            sprite_count: 0,
            sprite_patterns_lo: [0; 8],
            sprite_patterns_hi: [0; 8],
            sprite_positions: [0; 8],
            sprite_priorities: [0; 8],
            sprite_indexes: [0; 8],
            sprite_attributes: [0; 8],
            framebuffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            nmi_output: false,
            nmi_occurred: false,
            odd_frame: false,
            rendering_tables: optimized::RenderingTables::new(),
        }
    }
    
    pub fn read_register(&mut self, reg: u8) -> u8 {
        match reg {
            0 => self.open_bus, // PPUCTRL is write-only
            1 => self.open_bus, // PPUMASK is write-only
            2 => {
                // PPUSTATUS
                let value = (self.status & 0xE0) | (self.open_bus & 0x1F);
                self.status &= !0x80;  // Clear vblank flag
                self.w = false;         // Reset write toggle
                
                // VBlank suppression check
                if self.scanline == 241 && self.cycle == 0 {
                    self.suppress_vbl = true;
                }
                
                self.open_bus = value;
                value
            }
            3 => self.open_bus, // OAMADDR is write-only
            4 => {
                // OAMDATA
                let data = self.oam[self.oam_addr as usize];
                // Special case: reading during rendering returns 0xFF for sprite attributes
                if self.is_rendering() && self.cycle >= 1 && self.cycle <= 64 {
                    self.open_bus = 0xFF;
                    0xFF
                } else {
                    self.open_bus = data;
                    data
                }
            }
            5 => self.open_bus, // PPUSCROLL is write-only
            6 => self.open_bus, // PPUADDR is write-only
            7 => {
                // PPUDATA
                let addr = self.v & 0x3FFF;
                let mut value = self.buffer;
                
                // Palette reads are immediate
                if addr >= 0x3F00 {
                    let mut palette_addr = (addr & 0x1F) as usize;
                    if palette_addr >= 0x10 && palette_addr % 4 == 0 {
                        palette_addr &= 0x0F;
                    }
                    value = self.palette[palette_addr] & (if self.mask & 0x01 != 0 { 0x30 } else { 0x3F });
                    // Buffer gets nametable data at addr - 0x1000
                    if addr < 0x3F00 {
                        self.buffer = self.vram[self.mirror_address(addr - 0x1000) as usize];
                    }
                } else if addr >= 0x2000 {
                    self.buffer = self.vram[self.mirror_address(addr) as usize];
                } else {
                    // CHR reads need cartridge reference
                    self.buffer = self.open_bus; // Use open bus for now
                }
                
                self.v = self.v.wrapping_add(self.addr_increment());
                self.open_bus = value;
                value
            }
            _ => self.open_bus,
        }
    }
    
    pub fn write_register(&mut self, reg: u8, value: u8) {
        self.open_bus = value; // All writes update open bus
        
        match reg {
            0 => {
                self.ctrl = value;      // PPUCTRL
                // Update NMI output based on bit 7
                self.nmi_output = (value & 0x80) != 0;
                // Update temporary VRAM address nametable bits
                self.t = (self.t & 0xF3FF) | (((value as u16) & 0x03) << 10);
            }
            1 => self.mask = value,      // PPUMASK
            3 => self.oam_addr = value,  // OAMADDR
            4 => {
                // OAMDATA
                self.write_oam_byte(self.oam_addr, value);
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
                // Note: This is a simplified version - in real implementation,
                // we'd need the cartridge reference here
                let addr = self.v & 0x3FFF;
                if addr >= 0x3F00 {
                    // Palette write
                    let mut palette_addr = (addr & 0x1F) as usize;
                    if palette_addr >= 0x10 && palette_addr % 4 == 0 {
                        palette_addr &= 0x0F;
                    }
                    self.palette[palette_addr] = value;
                } else if addr >= 0x2000 {
                    // Name table write
                    let mirrored = self.mirror_address(addr);
                    self.vram[mirrored as usize] = value;
                }
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
    
    pub fn read_chr(&self, addr: u16, cartridge: &Cartridge) -> u8 {
        cartridge.read_chr(addr)
    }
    
    pub fn write_chr(&mut self, addr: u16, value: u8, cartridge: &mut Cartridge) {
        cartridge.write_chr(addr, value);
    }
    
    fn read_byte(&self, addr: u16, cartridge: &Cartridge) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables - read from cartridge CHR
                self.read_chr(addr, cartridge)
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
    
    fn write_byte(&mut self, addr: u16, value: u8, cartridge: &mut Cartridge) {
        match addr {
            0x0000..=0x1FFF => {
                // Pattern tables - write to cartridge CHR
                self.write_chr(addr, value, cartridge);
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
    
    pub fn step(&mut self, cartridge: &Cartridge) -> bool {
        // Visible scanlines (0-239)
        if self.scanline >= 0 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }
            
            if self.cycle >= 1 && self.cycle <= 256 {
                self.render_pixel(cartridge);
            }
            
            // Background fetch cycles
            if (self.cycle >= 1 && self.cycle <= 256) || (self.cycle >= 321 && self.cycle <= 336) {
                self.update_shifters();
                
                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();
                        self.bg_next_tile_id = self.fetch_nametable_byte(cartridge);
                    }
                    2 => {
                        self.bg_next_tile_attrib = self.fetch_attribute_byte(cartridge);
                    }
                    4 => {
                        self.bg_next_tile_lsb = self.fetch_pattern_byte(0, cartridge);
                    }
                    6 => {
                        self.bg_next_tile_msb = self.fetch_pattern_byte(1, cartridge);
                    }
                    7 => {
                        self.increment_x();
                    }
                    _ => {}
                }
            }
            
            if self.cycle == 256 {
                self.increment_y();
            }
            
            if self.cycle == 257 {
                self.load_background_shifters();
                if self.mask & 0x08 != 0 {
                    self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
                }
                
                // Sprite evaluation for next scanline
                if self.scanline >= -1 && self.scanline < 239 {
                    // Use optimized sprite evaluation if rendering is enabled
                    if self.is_rendering() {
                        optimized::evaluate_sprites_fast(self, self.scanline + 1);
                    } else {
                        self.evaluate_sprites(self.scanline + 1);
                    }
                }
            }
            
            if self.cycle == 338 || self.cycle == 340 {
                self.fetch_nametable_byte(cartridge);
            }
            
            // Fetch sprite data
            if self.cycle >= 257 && self.cycle <= 320 {
                self.fetch_sprite_data(cartridge);
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
                // Check for VBlank suppression
                if !self.suppress_vbl {
                    self.status |= 0x80;  // Set vblank flag
                    self.nmi_occurred = true;
                    if self.nmi_output {
                        // NMI will be triggered
                    }
                }
                self.suppress_vbl = false; // Reset suppression flag
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
    
    fn render_pixel(&mut self, _cartridge: &Cartridge) {
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;
        
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            let pixel_offset = y * SCREEN_WIDTH + x;
            
            let mut bg_pixel = 0;
            let mut bg_palette = 0;
            
            // Background rendering enabled?
            if self.mask & 0x08 != 0 {
                // Get pixel from shift registers
                let bit_mux = 0x8000 >> self.x;
                let p0_pixel = ((self.bg_shift_pattern_lo & bit_mux) > 0) as u8;
                let p1_pixel = ((self.bg_shift_pattern_hi & bit_mux) > 0) as u8;
                bg_pixel = (p1_pixel << 1) | p0_pixel;
                
                let bg_pal0 = ((self.bg_shift_attrib_lo & bit_mux) > 0) as u8;
                let bg_pal1 = ((self.bg_shift_attrib_hi & bit_mux) > 0) as u8;
                bg_palette = (bg_pal1 << 1) | bg_pal0;
            }
            
            // Check sprite rendering
            let mut sprite_pixel = 0;
            let mut sprite_palette = 0;
            let mut sprite_priority = false;
            let mut sprite_zero_hit = false;
            
            if self.mask & 0x10 != 0 {
                // Sprites enabled
                for i in 0..self.sprite_count {
                    let x_diff = (x as i32) - (self.sprite_positions[i as usize] as i32);
                    if x_diff >= 0 && x_diff < 8 {
                        let sprite_lo = self.sprite_patterns_lo[i as usize];
                        let sprite_hi = self.sprite_patterns_hi[i as usize];
                        let attr = self.sprite_attributes[i as usize];
                        
                        let mut bit = 7 - x_diff;
                        if attr & 0x40 != 0 {
                            // Horizontal flip
                            bit = x_diff;
                        }
                        
                        let lo = ((sprite_lo >> bit) & 1) as u8;
                        let hi = ((sprite_hi >> bit) & 1) as u8;
                        let pixel = (hi << 1) | lo;
                        
                        if pixel != 0 {
                            if i == 0 && self.sprite_indexes[0] == 0 && bg_pixel != 0 {
                                // Sprite 0 hit
                                sprite_zero_hit = true;
                            }
                            
                            if sprite_pixel == 0 {
                                sprite_pixel = pixel;
                                sprite_palette = (attr & 0x03) + 4;
                                sprite_priority = attr & 0x20 == 0;
                            }
                        }
                    }
                }
            }
            
            // Determine final pixel
            let (final_pixel, final_palette) = if bg_pixel == 0 && sprite_pixel == 0 {
                (0, 0)
            } else if bg_pixel == 0 && sprite_pixel != 0 {
                (sprite_pixel, sprite_palette)
            } else if bg_pixel != 0 && sprite_pixel == 0 {
                (bg_pixel, bg_palette)
            } else {
                // Both background and sprite are visible
                if sprite_priority {
                    (sprite_pixel, sprite_palette)
                } else {
                    (bg_pixel, bg_palette)
                }
            };
            
            if sprite_zero_hit && x < 255 {
                self.status |= 0x40; // Set sprite 0 hit flag
            }
            let palette_addr = if final_pixel == 0 {
                0  // Universal background color
            } else {
                (final_palette << 2) | final_pixel
            };
            
            let palette_index = self.palette[palette_addr as usize] & 0x3F;
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
        self.bg_shift_pattern_lo = (self.bg_shift_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shift_pattern_hi = (self.bg_shift_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;
        
        self.bg_shift_attrib_lo = (self.bg_shift_attrib_lo & 0xFF00) | (if self.bg_next_tile_attrib & 0x01 != 0 { 0xFF } else { 0x00 });
        self.bg_shift_attrib_hi = (self.bg_shift_attrib_hi & 0xFF00) | (if self.bg_next_tile_attrib & 0x02 != 0 { 0xFF } else { 0x00 });
    }
    
    fn update_shifters(&mut self) {
        if self.mask & 0x08 != 0 {
            self.bg_shift_pattern_lo <<= 1;
            self.bg_shift_pattern_hi <<= 1;
            self.bg_shift_attrib_lo <<= 1;
            self.bg_shift_attrib_hi <<= 1;
        }
    }
    
    fn fetch_nametable_byte(&self, cartridge: &Cartridge) -> u8 {
        let addr = 0x2000 | (self.v & 0x0FFF);
        self.read_byte(addr, cartridge)
    }
    
    fn fetch_attribute_byte(&self, cartridge: &Cartridge) -> u8 {
        let v = self.v;
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let shift = ((v >> 4) & 4) | (v & 2);
        let attrib = self.read_byte(addr, cartridge);
        (attrib >> shift) & 0x03
    }
    
    fn fetch_pattern_byte(&self, plane: u8, cartridge: &Cartridge) -> u8 {
        let fine_y = (self.v >> 12) & 0x07;
        let table = if self.ctrl & 0x10 != 0 { 0x1000 } else { 0x0000 };
        let addr = table | ((self.bg_next_tile_id as u16) << 4) | ((plane as u16) << 3) | fine_y;
        self.read_byte(addr, cartridge)
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
    
    pub fn get_ctrl(&self) -> u8 {
        self.ctrl
    }
    
    fn is_rendering(&self) -> bool {
        (self.mask & 0x18) != 0
    }
    
    pub fn write_oam_byte(&mut self, addr: u8, value: u8) {
        self.oam[addr as usize] = value;
        
        // OAM corruption during rendering
        if self.is_rendering() && self.cycle >= 1 && self.cycle <= 64 {
            // Writing to OAM during rendering can corrupt sprite data
            // This is a hardware bug where the first 8 bytes get corrupted
            for i in 0..8 {
                self.oam[i] = (self.oam[i] & 0xE0) | ((addr >> 2) & 0x07);
            }
        }
    }
    
    fn evaluate_sprites(&mut self, scanline: i32) {
        self.sprite_count = 0;
        
        let sprite_height = if self.ctrl & 0x20 != 0 { 16 } else { 8 };
        
        // Clear secondary OAM
        for i in 0..8 {
            self.sprite_patterns_lo[i] = 0;
            self.sprite_patterns_hi[i] = 0;
            self.sprite_positions[i] = 0xFF;
            self.sprite_priorities[i] = 0;
            self.sprite_indexes[i] = 0xFF;
            self.sprite_attributes[i] = 0;
        }
        
        // Clear secondary OAM with 0xFF
        for i in 0..32 {
            self.secondary_oam[i] = 0xFF;
        }
        
        let mut n = 0; // OAM index
        let mut m = 0; // OAM byte index
        
        // Sprite evaluation with accurate overflow behavior
        while n < 64 && self.sprite_count < 8 {
            let y = self.oam[n * 4] as i32;
            
            // Check if sprite is in range
            let y_diff = scanline - y;
            if y_diff >= 0 && y_diff < sprite_height {
                // Copy sprite to secondary OAM
                let idx = self.sprite_count as usize;
                for i in 0..4 {
                    self.secondary_oam[idx * 4 + i] = self.oam[n * 4 + i];
                }
                
                self.sprite_positions[idx] = self.oam[n * 4 + 3];
                self.sprite_attributes[idx] = self.oam[n * 4 + 2];
                self.sprite_indexes[idx] = n as u8;
                
                self.sprite_count += 1;
            }
            n += 1;
        }
        
        // Sprite overflow evaluation (with hardware bug emulation)
        if self.sprite_count == 8 && n < 64 {
            // Continue evaluation with buggy behavior
            while n < 64 {
                let y = self.oam[n * 4 + m] as i32;
                let y_diff = scanline - y;
                
                if y_diff >= 0 && y_diff < sprite_height {
                    // Set sprite overflow flag
                    self.status |= 0x20;
                    break;
                }
                
                // Increment with overflow bug
                m = (m + 1) & 3;
                if m == 0 {
                    n += 1;
                }
            }
        }
    }
    
    fn fetch_sprite_data(&mut self, cartridge: &Cartridge) {
        if self.cycle >= 257 && self.cycle < 321 {
            let sprite_idx = ((self.cycle - 257) / 8) as usize;
            
            if sprite_idx < self.sprite_count as usize {
                let cycle_in_fetch = (self.cycle - 257) % 8;
                
                if cycle_in_fetch == 4 {
                    // Fetch low sprite byte
                    let y = self.secondary_oam[sprite_idx * 4] as i32;
                    let tile = self.secondary_oam[sprite_idx * 4 + 1];
                    let attr = self.secondary_oam[sprite_idx * 4 + 2];
                    
                    let sprite_height = if self.ctrl & 0x20 != 0 { 16 } else { 8 };
                    let y_in_tile = (self.scanline - y) as u16;
                    
                    let mut y_offset = y_in_tile;
                    if attr & 0x80 != 0 {
                        // Vertical flip
                        y_offset = (sprite_height - 1) as u16 - y_offset;
                    }
                    
                    let pattern_addr = if sprite_height == 8 {
                        let table = if self.ctrl & 0x08 != 0 { 0x1000 } else { 0x0000 };
                        table | ((tile as u16) << 4) | y_offset
                    } else {
                        // 8x16 sprites
                        let table = ((tile & 1) as u16) << 12;
                        let tile_num = (tile & 0xFE) as u16;
                        if y_offset >= 8 {
                            table | ((tile_num + 1) << 4) | (y_offset - 8)
                        } else {
                            table | (tile_num << 4) | y_offset
                        }
                    };
                    
                    self.sprite_patterns_lo[sprite_idx] = self.read_byte(pattern_addr, cartridge);
                } else if cycle_in_fetch == 6 {
                    // Fetch high sprite byte
                    let y = self.secondary_oam[sprite_idx * 4] as i32;
                    let tile = self.secondary_oam[sprite_idx * 4 + 1];
                    let attr = self.secondary_oam[sprite_idx * 4 + 2];
                    
                    let sprite_height = if self.ctrl & 0x20 != 0 { 16 } else { 8 };
                    let y_in_tile = (self.scanline - y) as u16;
                    
                    let mut y_offset = y_in_tile;
                    if attr & 0x80 != 0 {
                        // Vertical flip
                        y_offset = (sprite_height - 1) as u16 - y_offset;
                    }
                    
                    let pattern_addr = if sprite_height == 8 {
                        let table = if self.ctrl & 0x08 != 0 { 0x1000 } else { 0x0000 };
                        table | ((tile as u16) << 4) | y_offset | 8
                    } else {
                        // 8x16 sprites  
                        let table = ((tile & 1) as u16) << 12;
                        let tile_num = (tile & 0xFE) as u16;
                        if y_offset >= 8 {
                            table | ((tile_num + 1) << 4) | (y_offset - 8) | 8
                        } else {
                            table | (tile_num << 4) | y_offset | 8
                        }
                    };
                    
                    self.sprite_patterns_hi[sprite_idx] = self.read_byte(pattern_addr, cartridge);
                }
            }
        }
    }
}
/// Optimized PPU rendering helpers

use super::*;

/// Precomputed tables for faster rendering
#[derive(Debug, Clone)]
pub struct RenderingTables {
    /// Attribute table shift lookup
    pub attribute_shift: [[u8; 4]; 4],
    /// Tile address lookup
    pub tile_address: [u16; 512],
}

impl RenderingTables {
    pub fn new() -> Self {
        let mut attribute_shift = [[0; 4]; 4];
        for y in 0..4 {
            for x in 0..4 {
                attribute_shift[y][x] = (((y & 2) << 1) | (x & 2)) as u8;
            }
        }
        
        let mut tile_address = [0; 512];
        for i in 0..512 {
            tile_address[i] = (i as u16) << 4;
        }
        
        Self {
            attribute_shift,
            tile_address,
        }
    }
}

/// Fast pixel rendering with minimal checks
#[inline(always)]
pub fn render_background_pixel_fast(
    ppu: &Ppu,
    _x: usize,
    _tables: &RenderingTables,
) -> (u8, u8) {
    if ppu.mask & 0x08 == 0 {
        return (0, 0);
    }
    
    // Get pixel from shift registers
    let bit_mux = 0x8000 >> ppu.x;
    let p0_pixel = ((ppu.bg_shift_pattern_lo & bit_mux) > 0) as u8;
    let p1_pixel = ((ppu.bg_shift_pattern_hi & bit_mux) > 0) as u8;
    let pixel = (p1_pixel << 1) | p0_pixel;
    
    let bg_pal0 = ((ppu.bg_shift_attrib_lo & bit_mux) > 0) as u8;
    let bg_pal1 = ((ppu.bg_shift_attrib_hi & bit_mux) > 0) as u8;
    let palette = (bg_pal1 << 1) | bg_pal0;
    
    (pixel, palette)
}

/// Optimized sprite evaluation
#[inline(always)]
pub fn evaluate_sprites_fast(
    ppu: &mut Ppu,
    scanline: i32,
) {
    ppu.sprite_count = 0;
    let sprite_height = if ppu.ctrl & 0x20 != 0 { 16 } else { 8 };
    
    // Clear sprite data
    unsafe {
        // Use unsafe for performance - we know these arrays are 8 elements
        std::ptr::write_bytes(ppu.sprite_patterns_lo.as_mut_ptr(), 0, 8);
        std::ptr::write_bytes(ppu.sprite_patterns_hi.as_mut_ptr(), 0, 8);
        std::ptr::write_bytes(ppu.sprite_positions.as_mut_ptr(), 0xFF, 8);
        std::ptr::write_bytes(ppu.sprite_priorities.as_mut_ptr(), 0, 8);
        std::ptr::write_bytes(ppu.sprite_indexes.as_mut_ptr(), 0xFF, 8);
        std::ptr::write_bytes(ppu.sprite_attributes.as_mut_ptr(), 0, 8);
    }
    
    // Fast sprite evaluation loop
    let mut oam_offset = 0;
    while oam_offset < 256 && ppu.sprite_count < 8 {
        let y = ppu.oam[oam_offset] as i32;
        let y_diff = scanline - y;
        
        if y_diff >= 0 && y_diff < sprite_height {
            let idx = ppu.sprite_count as usize;
            
            // Copy sprite data
            unsafe {
                std::ptr::copy_nonoverlapping(
                    ppu.oam.as_ptr().add(oam_offset),
                    ppu.secondary_oam.as_mut_ptr().add(idx * 4),
                    4
                );
            }
            
            ppu.sprite_positions[idx] = ppu.oam[oam_offset + 3];
            ppu.sprite_attributes[idx] = ppu.oam[oam_offset + 2];
            ppu.sprite_indexes[idx] = (oam_offset / 4) as u8;
            
            ppu.sprite_count += 1;
        }
        
        oam_offset += 4;
    }
    
    // Check for sprite overflow
    if ppu.sprite_count == 8 && oam_offset < 256 {
        while oam_offset < 256 {
            let y = ppu.oam[oam_offset] as i32;
            let y_diff = scanline - y;
            
            if y_diff >= 0 && y_diff < sprite_height {
                ppu.status |= 0x20; // Set sprite overflow
                break;
            }
            
            oam_offset += 4;
        }
    }
}

/// Batch pixel rendering for better cache efficiency
#[inline(always)]
pub fn render_scanline_batch(
    _ppu: &mut Ppu,
    scanline: usize,
    framebuffer: &mut [u32],
    palette: &[u32; 64],
) {
    if scanline >= SCREEN_HEIGHT {
        return;
    }
    
    let scanline_offset = scanline * SCREEN_WIDTH;
    
    // Process pixels in groups of 8 for better cache usage
    for x in (0..SCREEN_WIDTH).step_by(8) {
        for offset in 0..8 {
            let x_pos = x + offset;
            if x_pos >= SCREEN_WIDTH {
                break;
            }
            
            // Simplified pixel rendering - actual implementation would use
            // proper background and sprite mixing
            let palette_index = ((x_pos / 8) + scanline) & 0x3F;
            framebuffer[scanline_offset + x_pos] = palette[palette_index];
        }
    }
}

/// Optimized pattern fetch
#[inline(always)]
pub fn fetch_tile_pattern_fast(
    pattern_table_addr: u16,
    tile_id: u8,
    fine_y: u16,
    plane: u8,
) -> u8 {
    let addr = pattern_table_addr 
        | ((tile_id as u16) << 4) 
        | ((plane as u16) << 3) 
        | fine_y;
    
    // In real implementation, this would read from CHR ROM/RAM
    // For now, return a pattern
    ((addr >> 2) ^ (addr >> 4)) as u8
}

/// Fast attribute byte calculation
#[inline(always)]
pub fn get_attribute_fast(
    v: u16,
    tables: &RenderingTables,
) -> u8 {
    let coarse_x = (v & 0x001F) >> 2;
    let coarse_y = ((v & 0x03E0) >> 5) >> 2;
    let shift = tables.attribute_shift[coarse_y as usize & 3][coarse_x as usize & 3];
    
    // Simplified attribute calculation
    shift
}
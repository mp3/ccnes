/// Optimized CPU execution helpers

use super::*;

/// Optimized memory read with inline caching hints
#[inline(always)]
pub fn read_byte_fast(bus: &mut impl CpuBus, addr: u16) -> u8 {
    // Most common case: ROM reads
    if addr >= 0x8000 {
        return bus.read(addr);
    }
    
    // Second most common: Zero page
    if addr < 0x100 {
        return bus.read(addr);
    }
    
    // Everything else
    bus.read(addr)
}

/// Optimized memory write
#[inline(always)]
pub fn write_byte_fast(bus: &mut impl CpuBus, addr: u16, value: u8) {
    bus.write(addr, value);
}

/// Fast zero page read
#[inline(always)]
pub fn read_zero_page(bus: &mut impl CpuBus, addr: u8) -> u8 {
    bus.read(addr as u16)
}

/// Fast zero page write
#[inline(always)]
pub fn write_zero_page(bus: &mut impl CpuBus, addr: u8, value: u8) {
    bus.write(addr as u16, value);
}

/// Optimized 16-bit address read
#[inline(always)]
pub fn read_word(bus: &mut impl CpuBus, addr: u16) -> u16 {
    let lo = bus.read(addr) as u16;
    let hi = bus.read(addr.wrapping_add(1)) as u16;
    (hi << 8) | lo
}

/// Optimized zero page word read
#[inline(always)]
pub fn read_zero_page_word(bus: &mut impl CpuBus, addr: u8) -> u16 {
    let lo = bus.read(addr as u16) as u16;
    let hi = bus.read(addr.wrapping_add(1) as u16) as u16;
    (hi << 8) | lo
}

/// Page boundary crossing check
#[inline(always)]
pub fn page_crossed(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xFF00) != (addr2 & 0xFF00)
}

/// Optimized branch helper
#[inline(always)]
pub fn branch_relative(pc: &mut u16, offset: i8, cycles: &mut u32) {
    let old_pc = *pc;
    *pc = pc.wrapping_add(offset as u16);
    *cycles += 1;
    
    if page_crossed(old_pc, *pc) {
        *cycles += 1;
    }
}

/// Fast flag operations
pub mod flags {
    use super::*;
    
    #[inline(always)]
    pub fn set_nz(cpu: &mut Cpu, value: u8) {
        cpu.status.set(StatusFlags::ZERO, value == 0);
        cpu.status.set(StatusFlags::NEGATIVE, value & 0x80 != 0);
    }
    
    #[inline(always)]
    pub fn set_carry(cpu: &mut Cpu, value: bool) {
        cpu.status.set(StatusFlags::CARRY, value);
    }
    
    #[inline(always)]
    pub fn set_overflow(cpu: &mut Cpu, value: bool) {
        cpu.status.set(StatusFlags::OVERFLOW, value);
    }
}


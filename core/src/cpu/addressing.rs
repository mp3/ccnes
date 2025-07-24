use crate::cpu::{Cpu, CpuBus};

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

impl Cpu {
    pub fn get_operand_address(&mut self, mode: AddressingMode, bus: &mut impl CpuBus) -> (u16, bool) {
        let mut page_crossed = false;
        
        let addr = match mode {
            AddressingMode::Implicit => 0,
            
            AddressingMode::Accumulator => 0,
            
            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                addr
            }
            
            AddressingMode::ZeroPage => {
                let addr = bus.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                addr
            }
            
            AddressingMode::ZeroPageX => {
                let base = bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                base.wrapping_add(self.x) as u16
            }
            
            AddressingMode::ZeroPageY => {
                let base = bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                base.wrapping_add(self.y) as u16
            }
            
            AddressingMode::Absolute => {
                let lo = bus.read(self.pc) as u16;
                let hi = bus.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                (hi << 8) | lo
            }
            
            AddressingMode::AbsoluteX => {
                let lo = bus.read(self.pc) as u16;
                let hi = bus.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                let base = (hi << 8) | lo;
                let addr = base.wrapping_add(self.x as u16);
                page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                addr
            }
            
            AddressingMode::AbsoluteY => {
                let lo = bus.read(self.pc) as u16;
                let hi = bus.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                let base = (hi << 8) | lo;
                let addr = base.wrapping_add(self.y as u16);
                page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                addr
            }
            
            AddressingMode::Indirect => {
                let ptr_lo = bus.read(self.pc) as u16;
                let ptr_hi = bus.read(self.pc.wrapping_add(1)) as u16;
                self.pc = self.pc.wrapping_add(2);
                let ptr = (ptr_hi << 8) | ptr_lo;
                
                // 6502 bug: if low byte is 0xFF, high byte is fetched from beginning of page
                let lo = bus.read(ptr) as u16;
                let hi = if ptr_lo == 0xFF {
                    bus.read(ptr & 0xFF00) as u16
                } else {
                    bus.read(ptr.wrapping_add(1)) as u16
                };
                (hi << 8) | lo
            }
            
            AddressingMode::IndirectX => {
                let base = bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let ptr = base.wrapping_add(self.x);
                let lo = bus.read(ptr as u16) as u16;
                let hi = bus.read(ptr.wrapping_add(1) as u16) as u16;
                (hi << 8) | lo
            }
            
            AddressingMode::IndirectY => {
                let base = bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let lo = bus.read(base as u16) as u16;
                let hi = bus.read(base.wrapping_add(1) as u16) as u16;
                let base_addr = (hi << 8) | lo;
                let addr = base_addr.wrapping_add(self.y as u16);
                page_crossed = (base_addr & 0xFF00) != (addr & 0xFF00);
                addr
            }
            
            AddressingMode::Relative => {
                let offset = bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.pc.wrapping_add(offset as i8 as u16)
            }
        };
        
        (addr, page_crossed)
    }
}
use crate::cpu::{Cpu, CpuBus, StatusFlags};
use super::addressing::AddressingMode;

impl Cpu {
    // Load operations
    pub fn lda(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        self.a = self.read_byte(bus, addr, mode);
        self.set_zn_flags(self.a);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn ldx(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        self.x = self.read_byte(bus, addr, mode);
        self.set_zn_flags(self.x);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn ldy(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        self.y = self.read_byte(bus, addr, mode);
        self.set_zn_flags(self.y);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    // Store operations
    pub fn sta(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        self.write_byte(bus, addr, self.a, mode);
    }
    
    pub fn stx(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        self.write_byte(bus, addr, self.x, mode);
    }
    
    pub fn sty(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        self.write_byte(bus, addr, self.y, mode);
    }
    
    // Transfer operations
    pub fn tax(&mut self) {
        self.x = self.a;
        self.set_zn_flags(self.x);
    }
    
    pub fn tay(&mut self) {
        self.y = self.a;
        self.set_zn_flags(self.y);
    }
    
    pub fn txa(&mut self) {
        self.a = self.x;
        self.set_zn_flags(self.a);
    }
    
    pub fn tya(&mut self) {
        self.a = self.y;
        self.set_zn_flags(self.a);
    }
    
    pub fn tsx(&mut self) {
        self.x = self.sp;
        self.set_zn_flags(self.x);
    }
    
    pub fn txs(&mut self) {
        self.sp = self.x;
    }
    
    // Stack operations
    pub fn pha(&mut self, bus: &mut impl CpuBus) {
        self.push(self.a, bus);
    }
    
    pub fn php(&mut self, bus: &mut impl CpuBus) {
        self.push(self.status.bits() | StatusFlags::BREAK.bits() | StatusFlags::UNUSED.bits(), bus);
    }
    
    pub fn pla(&mut self, bus: &mut impl CpuBus) {
        self.a = self.pop(bus);
        self.set_zn_flags(self.a);
    }
    
    pub fn plp(&mut self, bus: &mut impl CpuBus) {
        let value = self.pop(bus);
        self.status = StatusFlags::from_bits_truncate(value) | StatusFlags::UNUSED;
        self.status.remove(StatusFlags::BREAK);
    }
    
    // Logical operations
    pub fn and(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        self.a &= value;
        self.set_zn_flags(self.a);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn eor(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        self.a ^= value;
        self.set_zn_flags(self.a);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn ora(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        self.a |= value;
        self.set_zn_flags(self.a);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn bit(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        let result = self.a & value;
        
        self.status.set(StatusFlags::ZERO, result == 0);
        self.status.set(StatusFlags::OVERFLOW, value & 0x40 != 0);
        self.status.set(StatusFlags::NEGATIVE, value & 0x80 != 0);
    }
    
    // Arithmetic operations
    pub fn adc(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        let carry = if self.status.contains(StatusFlags::CARRY) { 1 } else { 0 };
        
        let sum = self.a as u16 + value as u16 + carry;
        let result = sum as u8;
        
        self.status.set(StatusFlags::CARRY, sum > 0xFF);
        self.status.set(StatusFlags::OVERFLOW, 
            (self.a ^ result) & (value ^ result) & 0x80 != 0);
        
        self.a = result;
        self.set_zn_flags(self.a);
        
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn sbc(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        let borrow = if self.status.contains(StatusFlags::CARRY) { 0 } else { 1 };
        
        let diff = self.a as i16 - value as i16 - borrow;
        let result = diff as u8;
        
        self.status.set(StatusFlags::CARRY, diff >= 0);
        self.status.set(StatusFlags::OVERFLOW,
            (self.a ^ result) & (self.a ^ value) & 0x80 != 0);
        
        self.a = result;
        self.set_zn_flags(self.a);
        
        if page_crossed {
            self.add_cycle();
        }
    }
    
    // Compare operations
    pub fn cmp(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        self.compare(self.a, value);
        if page_crossed {
            self.add_cycle();
        }
    }
    
    pub fn cpx(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        self.compare(self.x, value);
    }
    
    pub fn cpy(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode);
        self.compare(self.y, value);
    }
    
    // Increment/Decrement operations
    pub fn inc(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode).wrapping_add(1);
        self.write_byte(bus, addr, value, mode);
        self.set_zn_flags(value);
    }
    
    pub fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_zn_flags(self.x);
    }
    
    pub fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_zn_flags(self.y);
    }
    
    pub fn dec(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let (addr, _) = self.get_operand_address(mode, bus);
        let value = self.read_byte(bus, addr, mode).wrapping_sub(1);
        self.write_byte(bus, addr, value, mode);
        self.set_zn_flags(value);
    }
    
    pub fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_zn_flags(self.x);
    }
    
    pub fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_zn_flags(self.y);
    }
    
    // Shift operations
    pub fn asl(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        if matches!(mode, AddressingMode::Accumulator) {
            self.status.set(StatusFlags::CARRY, self.a & 0x80 != 0);
            self.a <<= 1;
            self.set_zn_flags(self.a);
        } else {
            let (addr, _) = self.get_operand_address(mode, bus);
            let value = self.read_byte(bus, addr, mode);
            self.status.set(StatusFlags::CARRY, value & 0x80 != 0);
            let result = value << 1;
            self.write_byte(bus, addr, result, mode);
            self.set_zn_flags(result);
        }
    }
    
    pub fn lsr(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        if matches!(mode, AddressingMode::Accumulator) {
            self.status.set(StatusFlags::CARRY, self.a & 0x01 != 0);
            self.a >>= 1;
            self.set_zn_flags(self.a);
        } else {
            let (addr, _) = self.get_operand_address(mode, bus);
            let value = self.read_byte(bus, addr, mode);
            self.status.set(StatusFlags::CARRY, value & 0x01 != 0);
            let result = value >> 1;
            self.write_byte(bus, addr, result, mode);
            self.set_zn_flags(result);
        }
    }
    
    pub fn rol(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let old_carry = if self.status.contains(StatusFlags::CARRY) { 1 } else { 0 };
        
        if matches!(mode, AddressingMode::Accumulator) {
            self.status.set(StatusFlags::CARRY, self.a & 0x80 != 0);
            self.a = (self.a << 1) | old_carry;
            self.set_zn_flags(self.a);
        } else {
            let (addr, _) = self.get_operand_address(mode, bus);
            let value = self.read_byte(bus, addr, mode);
            self.status.set(StatusFlags::CARRY, value & 0x80 != 0);
            let result = (value << 1) | old_carry;
            self.write_byte(bus, addr, result, mode);
            self.set_zn_flags(result);
        }
    }
    
    pub fn ror(&mut self, bus: &mut impl CpuBus, mode: AddressingMode) {
        let old_carry = if self.status.contains(StatusFlags::CARRY) { 0x80 } else { 0 };
        
        if matches!(mode, AddressingMode::Accumulator) {
            self.status.set(StatusFlags::CARRY, self.a & 0x01 != 0);
            self.a = (self.a >> 1) | old_carry;
            self.set_zn_flags(self.a);
        } else {
            let (addr, _) = self.get_operand_address(mode, bus);
            let value = self.read_byte(bus, addr, mode);
            self.status.set(StatusFlags::CARRY, value & 0x01 != 0);
            let result = (value >> 1) | old_carry;
            self.write_byte(bus, addr, result, mode);
            self.set_zn_flags(result);
        }
    }
    
    // Jump operations
    pub fn jmp(&mut self, _bus: &mut impl CpuBus, _mode: AddressingMode, addr: u16) {
        self.pc = addr;
    }
    
    pub fn jsr(&mut self, bus: &mut impl CpuBus, addr: u16) {
        let return_addr = self.pc.wrapping_sub(1);
        self.push_word(return_addr, bus);
        self.pc = addr;
    }
    
    pub fn rts(&mut self, bus: &mut impl CpuBus) {
        let return_addr = self.pop_word(bus);
        self.pc = return_addr.wrapping_add(1);
    }
    
    pub fn rti(&mut self, bus: &mut impl CpuBus) {
        self.plp(bus);
        self.pc = self.pop_word(bus);
    }
    
    // Branch operations
    pub fn branch(&mut self, condition: bool, offset: i8) {
        if condition {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            self.add_cycle();
            
            // Add another cycle if page boundary crossed
            if (old_pc & 0xFF00) != (self.pc & 0xFF00) {
                self.add_cycle();
            }
        }
    }
    
    pub fn bcc(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(!self.status.contains(StatusFlags::CARRY), offset);
    }
    
    pub fn bcs(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(self.status.contains(StatusFlags::CARRY), offset);
    }
    
    pub fn beq(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(self.status.contains(StatusFlags::ZERO), offset);
    }
    
    pub fn bne(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(!self.status.contains(StatusFlags::ZERO), offset);
    }
    
    pub fn bmi(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(self.status.contains(StatusFlags::NEGATIVE), offset);
    }
    
    pub fn bpl(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(!self.status.contains(StatusFlags::NEGATIVE), offset);
    }
    
    pub fn bvc(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(!self.status.contains(StatusFlags::OVERFLOW), offset);
    }
    
    pub fn bvs(&mut self, bus: &mut impl CpuBus) {
        let offset = bus.read(self.pc.wrapping_sub(1)) as i8;
        self.branch(self.status.contains(StatusFlags::OVERFLOW), offset);
    }
    
    // Flag operations
    pub fn clc(&mut self) {
        self.status.remove(StatusFlags::CARRY);
    }
    
    pub fn cld(&mut self) {
        self.status.remove(StatusFlags::DECIMAL);
    }
    
    pub fn cli(&mut self) {
        self.status.remove(StatusFlags::INTERRUPT);
    }
    
    pub fn clv(&mut self) {
        self.status.remove(StatusFlags::OVERFLOW);
    }
    
    pub fn sec(&mut self) {
        self.status.insert(StatusFlags::CARRY);
    }
    
    pub fn sed(&mut self) {
        self.status.insert(StatusFlags::DECIMAL);
    }
    
    pub fn sei(&mut self) {
        self.status.insert(StatusFlags::INTERRUPT);
    }
    
    // Helper functions
    fn set_zn_flags(&mut self, value: u8) {
        self.status.set(StatusFlags::ZERO, value == 0);
        self.status.set(StatusFlags::NEGATIVE, value & 0x80 != 0);
    }
    
    fn compare(&mut self, a: u8, b: u8) {
        let result = a.wrapping_sub(b);
        self.status.set(StatusFlags::CARRY, a >= b);
        self.status.set(StatusFlags::ZERO, a == b);
        self.status.set(StatusFlags::NEGATIVE, result & 0x80 != 0);
    }
    
    fn read_byte(&self, bus: &mut impl CpuBus, addr: u16, mode: AddressingMode) -> u8 {
        if matches!(mode, AddressingMode::Immediate) {
            bus.read(addr)
        } else {
            bus.read(addr)
        }
    }
    
    fn write_byte(&self, bus: &mut impl CpuBus, addr: u16, value: u8, _mode: AddressingMode) {
        bus.write(addr, value);
    }
    
    fn pop_word(&mut self, bus: &mut impl CpuBus) -> u16 {
        let lo = self.pop(bus) as u16;
        let hi = self.pop(bus) as u16;
        (hi << 8) | lo
    }
    
    fn add_cycle(&mut self) {
        self.cycles += 1;
    }
}
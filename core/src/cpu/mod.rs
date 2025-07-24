use bitflags::bitflags;

pub mod instructions;
pub mod addressing;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusFlags: u8 {
        const CARRY     = 0b00000001;  // C
        const ZERO      = 0b00000010;  // Z
        const INTERRUPT = 0b00000100;  // I
        const DECIMAL   = 0b00001000;  // D
        const BREAK     = 0b00010000;  // B
        const UNUSED    = 0b00100000;  // -
        const OVERFLOW  = 0b01000000;  // V
        const NEGATIVE  = 0b10000000;  // N
    }
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub a: u8,      // Accumulator
    pub x: u8,      // X index register
    pub y: u8,      // Y index register
    pub sp: u8,     // Stack pointer
    pub pc: u16,    // Program counter
    pub status: StatusFlags,
    
    cycles: u32,
    stall_cycles: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            status: StatusFlags::from_bits_truncate(0x24),
            cycles: 0,
            stall_cycles: 0,
        }
    }
    
    pub fn reset(&mut self, bus: &impl CpuBus) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = StatusFlags::from_bits_truncate(0x24);
        
        let lo = bus.read(0xFFFC) as u16;
        let hi = bus.read(0xFFFD) as u16;
        self.pc = (hi << 8) | lo;
        
        self.cycles = 0;
        self.stall_cycles = 0;
    }
    
    pub fn step(&mut self, bus: &mut impl CpuBus) -> u32 {
        if self.stall_cycles > 0 {
            self.stall_cycles -= 1;
            return 1;
        }
        
        let opcode = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        
        let start_cycles = self.cycles;
        self.execute_instruction(opcode, bus);
        
        self.cycles - start_cycles
    }
    
    fn execute_instruction(&mut self, opcode: u8, bus: &mut impl CpuBus) {
        // This will be implemented with all 56 instructions
        match opcode {
            0x00 => self.brk(bus),
            0xEA => self.nop(),
            _ => panic!("Unimplemented opcode: 0x{:02X}", opcode),
        }
    }
    
    fn brk(&mut self, bus: &mut impl CpuBus) {
        self.pc = self.pc.wrapping_add(1);
        self.push_word(self.pc, bus);
        self.push(self.status.bits() | 0x10, bus);
        self.status.insert(StatusFlags::INTERRUPT);
        self.pc = self.read_word(0xFFFE, bus);
        self.cycles += 7;
    }
    
    fn nop(&mut self) {
        self.cycles += 2;
    }
    
    fn push(&mut self, value: u8, bus: &mut impl CpuBus) {
        bus.write(0x0100 | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }
    
    fn push_word(&mut self, value: u16, bus: &mut impl CpuBus) {
        self.push((value >> 8) as u8, bus);
        self.push(value as u8, bus);
    }
    
    fn pop(&mut self, bus: &impl CpuBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x0100 | self.sp as u16)
    }
    
    fn read_word(&self, addr: u16, bus: &impl CpuBus) -> u16 {
        let lo = bus.read(addr) as u16;
        let hi = bus.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }
}

pub trait CpuBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}
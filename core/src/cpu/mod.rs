use bitflags::bitflags;

pub mod instructions;
pub mod addressing;
pub mod opcodes;

use opcodes::{OPCODE_TABLE, Instruction};

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
    
    pub cycles: u32,
    stall_cycles: u32,
    
    // Interrupt flags
    nmi_pending: bool,
    irq_pending: bool,
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
            nmi_pending: false,
            irq_pending: false,
        }
    }
    
    pub fn reset(&mut self, bus: &mut impl CpuBus) {
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
        self.nmi_pending = false;
        self.irq_pending = false;
    }
    
    pub fn step(&mut self, bus: &mut impl CpuBus) -> u32 {
        if self.stall_cycles > 0 {
            self.stall_cycles -= 1;
            return 1;
        }
        
        // Handle interrupts
        self.handle_interrupts(bus);
        
        let opcode = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        
        let start_cycles = self.cycles;
        self.execute_instruction(opcode, bus);
        
        self.cycles - start_cycles
    }
    
    fn execute_instruction(&mut self, opcode_byte: u8, bus: &mut impl CpuBus) {
        let opcode = match OPCODE_TABLE[opcode_byte as usize] {
            Some(op) => op,
            None => panic!("Invalid opcode: 0x{:02X}", opcode_byte),
        };
        
        self.cycles += opcode.cycles as u32;
        
        match opcode.instruction {
            // Load/Store Operations
            Instruction::LDA => self.lda(bus, opcode.mode),
            Instruction::LDX => self.ldx(bus, opcode.mode),
            Instruction::LDY => self.ldy(bus, opcode.mode),
            Instruction::STA => self.sta(bus, opcode.mode),
            Instruction::STX => self.stx(bus, opcode.mode),
            Instruction::STY => self.sty(bus, opcode.mode),
            
            // Register Transfers
            Instruction::TAX => self.tax(),
            Instruction::TAY => self.tay(),
            Instruction::TXA => self.txa(),
            Instruction::TYA => self.tya(),
            Instruction::TSX => self.tsx(),
            Instruction::TXS => self.txs(),
            
            // Stack Operations
            Instruction::PHA => self.pha(bus),
            Instruction::PHP => self.php(bus),
            Instruction::PLA => self.pla(bus),
            Instruction::PLP => self.plp(bus),
            
            // Logical Operations
            Instruction::AND => self.and(bus, opcode.mode),
            Instruction::EOR => self.eor(bus, opcode.mode),
            Instruction::ORA => self.ora(bus, opcode.mode),
            Instruction::BIT => self.bit(bus, opcode.mode),
            
            // Arithmetic Operations
            Instruction::ADC => self.adc(bus, opcode.mode),
            Instruction::SBC => self.sbc(bus, opcode.mode),
            Instruction::CMP => self.cmp(bus, opcode.mode),
            Instruction::CPX => self.cpx(bus, opcode.mode),
            Instruction::CPY => self.cpy(bus, opcode.mode),
            
            // Increments & Decrements
            Instruction::INC => self.inc(bus, opcode.mode),
            Instruction::INX => self.inx(),
            Instruction::INY => self.iny(),
            Instruction::DEC => self.dec(bus, opcode.mode),
            Instruction::DEX => self.dex(),
            Instruction::DEY => self.dey(),
            
            // Shifts
            Instruction::ASL => self.asl(bus, opcode.mode),
            Instruction::LSR => self.lsr(bus, opcode.mode),
            Instruction::ROL => self.rol(bus, opcode.mode),
            Instruction::ROR => self.ror(bus, opcode.mode),
            
            // Jumps & Calls
            Instruction::JMP => {
                let (addr, _) = self.get_operand_address(opcode.mode, bus);
                self.jmp(bus, opcode.mode, addr);
            },
            Instruction::JSR => {
                let (addr, _) = self.get_operand_address(opcode.mode, bus);
                self.jsr(bus, addr);
            },
            Instruction::RTS => self.rts(bus),
            Instruction::RTI => self.rti(bus),
            
            // Branches
            Instruction::BCC => self.bcc(bus),
            Instruction::BCS => self.bcs(bus),
            Instruction::BEQ => self.beq(bus),
            Instruction::BNE => self.bne(bus),
            Instruction::BMI => self.bmi(bus),
            Instruction::BPL => self.bpl(bus),
            Instruction::BVC => self.bvc(bus),
            Instruction::BVS => self.bvs(bus),
            
            // Status Flag Changes
            Instruction::CLC => self.clc(),
            Instruction::CLD => self.cld(),
            Instruction::CLI => self.cli(),
            Instruction::CLV => self.clv(),
            Instruction::SEC => self.sec(),
            Instruction::SED => self.sed(),
            Instruction::SEI => self.sei(),
            
            // System Functions
            Instruction::BRK => self.brk(bus),
            Instruction::NOP => self.nop(),
        }
    }
    
    pub fn brk(&mut self, bus: &mut impl CpuBus) {
        self.pc = self.pc.wrapping_add(1);
        self.push_word(self.pc, bus);
        self.push(self.status.bits() | StatusFlags::BREAK.bits() | StatusFlags::UNUSED.bits(), bus);
        self.status.insert(StatusFlags::INTERRUPT);
        self.pc = self.read_word(0xFFFE, bus);
    }
    
    pub fn nop(&mut self) {
        // No operation
    }
    
    pub fn push(&mut self, value: u8, bus: &mut impl CpuBus) {
        bus.write(0x0100 | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }
    
    pub fn push_word(&mut self, value: u16, bus: &mut impl CpuBus) {
        self.push((value >> 8) as u8, bus);
        self.push(value as u8, bus);
    }
    
    pub fn pop(&mut self, bus: &mut impl CpuBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x0100 | self.sp as u16)
    }
    
    pub fn read_word(&self, addr: u16, bus: &mut impl CpuBus) -> u16 {
        let lo = bus.read(addr) as u16;
        let hi = bus.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }
    
    pub fn trigger_nmi(&mut self) {
        self.nmi_pending = true;
    }
    
    pub fn trigger_irq(&mut self) {
        if !self.status.contains(StatusFlags::INTERRUPT) {
            self.irq_pending = true;
        }
    }
    
    fn handle_interrupts(&mut self, bus: &mut impl CpuBus) {
        if self.nmi_pending {
            self.nmi_pending = false;
            self.push_word(self.pc, bus);
            self.push(self.status.bits() & !StatusFlags::BREAK.bits() | StatusFlags::UNUSED.bits(), bus);
            self.status.insert(StatusFlags::INTERRUPT);
            self.pc = self.read_word(0xFFFA, bus);
            self.cycles += 7;
        } else if self.irq_pending && !self.status.contains(StatusFlags::INTERRUPT) {
            self.irq_pending = false;
            self.push_word(self.pc, bus);
            self.push(self.status.bits() & !StatusFlags::BREAK.bits() | StatusFlags::UNUSED.bits(), bus);
            self.status.insert(StatusFlags::INTERRUPT);
            self.pc = self.read_word(0xFFFE, bus);
            self.cycles += 7;
        }
    }
}

pub trait CpuBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}
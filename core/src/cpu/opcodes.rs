use super::addressing::AddressingMode;

#[derive(Debug, Clone, Copy)]
pub struct Opcode {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub cycles: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // Load/Store Operations
    LDA, LDX, LDY,
    STA, STX, STY,
    
    // Register Transfers
    TAX, TAY, TXA, TYA,
    TSX, TXS,
    
    // Stack Operations
    PHA, PHP, PLA, PLP,
    
    // Logical Operations
    AND, EOR, ORA, BIT,
    
    // Arithmetic Operations
    ADC, SBC,
    CMP, CPX, CPY,
    
    // Increments & Decrements
    INC, INX, INY,
    DEC, DEX, DEY,
    
    // Shifts
    ASL, LSR, ROL, ROR,
    
    // Jumps & Calls
    JMP, JSR, RTS, RTI,
    
    // Branches
    BCC, BCS, BEQ, BNE,
    BMI, BPL, BVC, BVS,
    
    // Status Flag Changes
    CLC, CLD, CLI, CLV,
    SEC, SED, SEI,
    
    // System Functions
    BRK, NOP,
}

pub const OPCODE_TABLE: [Option<Opcode>; 256] = {
    let mut table = [None; 256];
    
    // BRK
    table[0x00] = Some(Opcode { instruction: Instruction::BRK, mode: AddressingMode::Implicit, cycles: 7 });
    
    // ORA
    table[0x01] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0x05] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x09] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::Immediate, cycles: 2 });
    table[0x0D] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x11] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0x15] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0x19] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0x1D] = Some(Opcode { instruction: Instruction::ORA, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // ASL
    table[0x06] = Some(Opcode { instruction: Instruction::ASL, mode: AddressingMode::ZeroPage, cycles: 5 });
    table[0x0A] = Some(Opcode { instruction: Instruction::ASL, mode: AddressingMode::Accumulator, cycles: 2 });
    table[0x0E] = Some(Opcode { instruction: Instruction::ASL, mode: AddressingMode::Absolute, cycles: 6 });
    table[0x16] = Some(Opcode { instruction: Instruction::ASL, mode: AddressingMode::ZeroPageX, cycles: 6 });
    table[0x1E] = Some(Opcode { instruction: Instruction::ASL, mode: AddressingMode::AbsoluteX, cycles: 7 });
    
    // PHP, CLC
    table[0x08] = Some(Opcode { instruction: Instruction::PHP, mode: AddressingMode::Implicit, cycles: 3 });
    table[0x18] = Some(Opcode { instruction: Instruction::CLC, mode: AddressingMode::Implicit, cycles: 2 });
    
    // JSR
    table[0x20] = Some(Opcode { instruction: Instruction::JSR, mode: AddressingMode::Absolute, cycles: 6 });
    
    // AND
    table[0x21] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0x25] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x29] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::Immediate, cycles: 2 });
    table[0x2D] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x31] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0x35] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0x39] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0x3D] = Some(Opcode { instruction: Instruction::AND, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // BIT
    table[0x24] = Some(Opcode { instruction: Instruction::BIT, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x2C] = Some(Opcode { instruction: Instruction::BIT, mode: AddressingMode::Absolute, cycles: 4 });
    
    // ROL
    table[0x26] = Some(Opcode { instruction: Instruction::ROL, mode: AddressingMode::ZeroPage, cycles: 5 });
    table[0x2A] = Some(Opcode { instruction: Instruction::ROL, mode: AddressingMode::Accumulator, cycles: 2 });
    table[0x2E] = Some(Opcode { instruction: Instruction::ROL, mode: AddressingMode::Absolute, cycles: 6 });
    table[0x36] = Some(Opcode { instruction: Instruction::ROL, mode: AddressingMode::ZeroPageX, cycles: 6 });
    table[0x3E] = Some(Opcode { instruction: Instruction::ROL, mode: AddressingMode::AbsoluteX, cycles: 7 });
    
    // PLP, SEC
    table[0x28] = Some(Opcode { instruction: Instruction::PLP, mode: AddressingMode::Implicit, cycles: 4 });
    table[0x38] = Some(Opcode { instruction: Instruction::SEC, mode: AddressingMode::Implicit, cycles: 2 });
    
    // Branch instructions
    table[0x10] = Some(Opcode { instruction: Instruction::BPL, mode: AddressingMode::Relative, cycles: 2 });
    table[0x30] = Some(Opcode { instruction: Instruction::BMI, mode: AddressingMode::Relative, cycles: 2 });
    table[0x50] = Some(Opcode { instruction: Instruction::BVC, mode: AddressingMode::Relative, cycles: 2 });
    table[0x70] = Some(Opcode { instruction: Instruction::BVS, mode: AddressingMode::Relative, cycles: 2 });
    table[0x90] = Some(Opcode { instruction: Instruction::BCC, mode: AddressingMode::Relative, cycles: 2 });
    table[0xB0] = Some(Opcode { instruction: Instruction::BCS, mode: AddressingMode::Relative, cycles: 2 });
    table[0xD0] = Some(Opcode { instruction: Instruction::BNE, mode: AddressingMode::Relative, cycles: 2 });
    table[0xF0] = Some(Opcode { instruction: Instruction::BEQ, mode: AddressingMode::Relative, cycles: 2 });
    
    // RTI
    table[0x40] = Some(Opcode { instruction: Instruction::RTI, mode: AddressingMode::Implicit, cycles: 6 });
    
    // EOR
    table[0x41] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0x45] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x49] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::Immediate, cycles: 2 });
    table[0x4D] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x51] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0x55] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0x59] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0x5D] = Some(Opcode { instruction: Instruction::EOR, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // LSR
    table[0x46] = Some(Opcode { instruction: Instruction::LSR, mode: AddressingMode::ZeroPage, cycles: 5 });
    table[0x4A] = Some(Opcode { instruction: Instruction::LSR, mode: AddressingMode::Accumulator, cycles: 2 });
    table[0x4E] = Some(Opcode { instruction: Instruction::LSR, mode: AddressingMode::Absolute, cycles: 6 });
    table[0x56] = Some(Opcode { instruction: Instruction::LSR, mode: AddressingMode::ZeroPageX, cycles: 6 });
    table[0x5E] = Some(Opcode { instruction: Instruction::LSR, mode: AddressingMode::AbsoluteX, cycles: 7 });
    
    // PHA, CLI
    table[0x48] = Some(Opcode { instruction: Instruction::PHA, mode: AddressingMode::Implicit, cycles: 3 });
    table[0x58] = Some(Opcode { instruction: Instruction::CLI, mode: AddressingMode::Implicit, cycles: 2 });
    
    // JMP
    table[0x4C] = Some(Opcode { instruction: Instruction::JMP, mode: AddressingMode::Absolute, cycles: 3 });
    table[0x6C] = Some(Opcode { instruction: Instruction::JMP, mode: AddressingMode::Indirect, cycles: 5 });
    
    // RTS
    table[0x60] = Some(Opcode { instruction: Instruction::RTS, mode: AddressingMode::Implicit, cycles: 6 });
    
    // ADC
    table[0x61] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0x65] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x69] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::Immediate, cycles: 2 });
    table[0x6D] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x71] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0x75] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0x79] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0x7D] = Some(Opcode { instruction: Instruction::ADC, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // ROR
    table[0x66] = Some(Opcode { instruction: Instruction::ROR, mode: AddressingMode::ZeroPage, cycles: 5 });
    table[0x6A] = Some(Opcode { instruction: Instruction::ROR, mode: AddressingMode::Accumulator, cycles: 2 });
    table[0x6E] = Some(Opcode { instruction: Instruction::ROR, mode: AddressingMode::Absolute, cycles: 6 });
    table[0x76] = Some(Opcode { instruction: Instruction::ROR, mode: AddressingMode::ZeroPageX, cycles: 6 });
    table[0x7E] = Some(Opcode { instruction: Instruction::ROR, mode: AddressingMode::AbsoluteX, cycles: 7 });
    
    // PLA, SEI
    table[0x68] = Some(Opcode { instruction: Instruction::PLA, mode: AddressingMode::Implicit, cycles: 4 });
    table[0x78] = Some(Opcode { instruction: Instruction::SEI, mode: AddressingMode::Implicit, cycles: 2 });
    
    // STA
    table[0x81] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0x85] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x8D] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x91] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::IndirectY, cycles: 6 });
    table[0x95] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0x99] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::AbsoluteY, cycles: 5 });
    table[0x9D] = Some(Opcode { instruction: Instruction::STA, mode: AddressingMode::AbsoluteX, cycles: 5 });
    
    // STX
    table[0x86] = Some(Opcode { instruction: Instruction::STX, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x8E] = Some(Opcode { instruction: Instruction::STX, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x96] = Some(Opcode { instruction: Instruction::STX, mode: AddressingMode::ZeroPageY, cycles: 4 });
    
    // STY
    table[0x84] = Some(Opcode { instruction: Instruction::STY, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0x8C] = Some(Opcode { instruction: Instruction::STY, mode: AddressingMode::Absolute, cycles: 4 });
    table[0x94] = Some(Opcode { instruction: Instruction::STY, mode: AddressingMode::ZeroPageX, cycles: 4 });
    
    // DEY, TXA, TYA
    table[0x88] = Some(Opcode { instruction: Instruction::DEY, mode: AddressingMode::Implicit, cycles: 2 });
    table[0x8A] = Some(Opcode { instruction: Instruction::TXA, mode: AddressingMode::Implicit, cycles: 2 });
    table[0x98] = Some(Opcode { instruction: Instruction::TYA, mode: AddressingMode::Implicit, cycles: 2 });
    
    // TXS
    table[0x9A] = Some(Opcode { instruction: Instruction::TXS, mode: AddressingMode::Implicit, cycles: 2 });
    
    // LDY
    table[0xA0] = Some(Opcode { instruction: Instruction::LDY, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xA4] = Some(Opcode { instruction: Instruction::LDY, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xAC] = Some(Opcode { instruction: Instruction::LDY, mode: AddressingMode::Absolute, cycles: 4 });
    table[0xB4] = Some(Opcode { instruction: Instruction::LDY, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0xBC] = Some(Opcode { instruction: Instruction::LDY, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // LDA
    table[0xA1] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0xA5] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xA9] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xAD] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::Absolute, cycles: 4 });
    table[0xB1] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0xB5] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0xB9] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0xBD] = Some(Opcode { instruction: Instruction::LDA, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // LDX
    table[0xA2] = Some(Opcode { instruction: Instruction::LDX, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xA6] = Some(Opcode { instruction: Instruction::LDX, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xAE] = Some(Opcode { instruction: Instruction::LDX, mode: AddressingMode::Absolute, cycles: 4 });
    table[0xB6] = Some(Opcode { instruction: Instruction::LDX, mode: AddressingMode::ZeroPageY, cycles: 4 });
    table[0xBE] = Some(Opcode { instruction: Instruction::LDX, mode: AddressingMode::AbsoluteY, cycles: 4 });
    
    // TAY, TAX, TSX
    table[0xA8] = Some(Opcode { instruction: Instruction::TAY, mode: AddressingMode::Implicit, cycles: 2 });
    table[0xAA] = Some(Opcode { instruction: Instruction::TAX, mode: AddressingMode::Implicit, cycles: 2 });
    table[0xBA] = Some(Opcode { instruction: Instruction::TSX, mode: AddressingMode::Implicit, cycles: 2 });
    
    // CLD, CLV
    table[0xB8] = Some(Opcode { instruction: Instruction::CLV, mode: AddressingMode::Implicit, cycles: 2 });
    table[0xD8] = Some(Opcode { instruction: Instruction::CLD, mode: AddressingMode::Implicit, cycles: 2 });
    
    // CPY
    table[0xC0] = Some(Opcode { instruction: Instruction::CPY, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xC4] = Some(Opcode { instruction: Instruction::CPY, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xCC] = Some(Opcode { instruction: Instruction::CPY, mode: AddressingMode::Absolute, cycles: 4 });
    
    // CMP
    table[0xC1] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0xC5] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xC9] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xCD] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::Absolute, cycles: 4 });
    table[0xD1] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0xD5] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0xD9] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0xDD] = Some(Opcode { instruction: Instruction::CMP, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // DEC
    table[0xC6] = Some(Opcode { instruction: Instruction::DEC, mode: AddressingMode::ZeroPage, cycles: 5 });
    table[0xCE] = Some(Opcode { instruction: Instruction::DEC, mode: AddressingMode::Absolute, cycles: 6 });
    table[0xD6] = Some(Opcode { instruction: Instruction::DEC, mode: AddressingMode::ZeroPageX, cycles: 6 });
    table[0xDE] = Some(Opcode { instruction: Instruction::DEC, mode: AddressingMode::AbsoluteX, cycles: 7 });
    
    // INY, DEX, INX
    table[0xC8] = Some(Opcode { instruction: Instruction::INY, mode: AddressingMode::Implicit, cycles: 2 });
    table[0xCA] = Some(Opcode { instruction: Instruction::DEX, mode: AddressingMode::Implicit, cycles: 2 });
    table[0xE8] = Some(Opcode { instruction: Instruction::INX, mode: AddressingMode::Implicit, cycles: 2 });
    
    // SBC
    table[0xE1] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::IndirectX, cycles: 6 });
    table[0xE5] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xE9] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xED] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::Absolute, cycles: 4 });
    table[0xF1] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::IndirectY, cycles: 5 });
    table[0xF5] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::ZeroPageX, cycles: 4 });
    table[0xF9] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::AbsoluteY, cycles: 4 });
    table[0xFD] = Some(Opcode { instruction: Instruction::SBC, mode: AddressingMode::AbsoluteX, cycles: 4 });
    
    // CPX
    table[0xE0] = Some(Opcode { instruction: Instruction::CPX, mode: AddressingMode::Immediate, cycles: 2 });
    table[0xE4] = Some(Opcode { instruction: Instruction::CPX, mode: AddressingMode::ZeroPage, cycles: 3 });
    table[0xEC] = Some(Opcode { instruction: Instruction::CPX, mode: AddressingMode::Absolute, cycles: 4 });
    
    // INC
    table[0xE6] = Some(Opcode { instruction: Instruction::INC, mode: AddressingMode::ZeroPage, cycles: 5 });
    table[0xEE] = Some(Opcode { instruction: Instruction::INC, mode: AddressingMode::Absolute, cycles: 6 });
    table[0xF6] = Some(Opcode { instruction: Instruction::INC, mode: AddressingMode::ZeroPageX, cycles: 6 });
    table[0xFE] = Some(Opcode { instruction: Instruction::INC, mode: AddressingMode::AbsoluteX, cycles: 7 });
    
    // NOP, SED
    table[0xEA] = Some(Opcode { instruction: Instruction::NOP, mode: AddressingMode::Implicit, cycles: 2 });
    table[0xF8] = Some(Opcode { instruction: Instruction::SED, mode: AddressingMode::Implicit, cycles: 2 });
    
    table
};
use ccnes_core::cpu::{Cpu, CpuBus, StatusFlags};

// Mock bus for testing
struct MockBus {
    memory: [u8; 0x10000],
}

impl MockBus {
    fn new() -> Self {
        Self {
            memory: [0; 0x10000],
        }
    }
    
    fn load_program(&mut self, program: &[u8], start: u16) {
        for (i, &byte) in program.iter().enumerate() {
            self.memory[start as usize + i] = byte;
        }
        // Set reset vector
        self.memory[0xFFFC] = (start & 0xFF) as u8;
        self.memory[0xFFFD] = (start >> 8) as u8;
    }
}

impl CpuBus for MockBus {
    fn read(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}

// Helper function to create CPU with program
fn setup_cpu(program: &[u8]) -> (Cpu, MockBus) {
    let mut bus = MockBus::new();
    bus.load_program(program, 0x8000);
    let mut cpu = Cpu::new();
    cpu.reset(&mut bus);
    (cpu, bus)
}

#[test]
fn test_lda_immediate() {
    let program = [0xA9, 0x42]; // LDA #$42
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(StatusFlags::ZERO));
    assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
}

#[test]
fn test_lda_zero_flag() {
    let program = [0xA9, 0x00]; // LDA #$00
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(StatusFlags::ZERO));
    assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
}

#[test]
fn test_lda_negative_flag() {
    let program = [0xA9, 0x80]; // LDA #$80
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.status.contains(StatusFlags::ZERO));
    assert!(cpu.status.contains(StatusFlags::NEGATIVE));
}

#[test]
fn test_ldx_ldy() {
    let program = [
        0xA2, 0x10, // LDX #$10
        0xA0, 0x20, // LDY #$20
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0x10);
    
    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0x20);
}

#[test]
fn test_sta_zeropage() {
    let program = [
        0xA9, 0x42, // LDA #$42
        0x85, 0x10, // STA $10
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(bus.read(0x10), 0x42);
}

#[test]
fn test_adc_no_carry() {
    let program = [
        0xA9, 0x10, // LDA #$10
        0x69, 0x20, // ADC #$20
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x30);
    assert!(!cpu.status.contains(StatusFlags::CARRY));
    assert!(!cpu.status.contains(StatusFlags::ZERO));
    assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
}

#[test]
fn test_adc_with_carry() {
    let program = [
        0xA9, 0xFF, // LDA #$FF
        0x69, 0x01, // ADC #$01
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(StatusFlags::CARRY));
    assert!(cpu.status.contains(StatusFlags::ZERO));
}

#[test]
fn test_sbc() {
    let program = [
        0x38,       // SEC (set carry for subtraction)
        0xA9, 0x50, // LDA #$50
        0xE9, 0x20, // SBC #$20
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x30);
    assert!(cpu.status.contains(StatusFlags::CARRY)); // No borrow
}

#[test]
fn test_and() {
    let program = [
        0xA9, 0xFF, // LDA #$FF
        0x29, 0x0F, // AND #$0F
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0x0F);
}

#[test]
fn test_ora() {
    let program = [
        0xA9, 0xF0, // LDA #$F0
        0x09, 0x0F, // ORA #$0F
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0xFF);
}

#[test]
fn test_eor() {
    let program = [
        0xA9, 0xFF, // LDA #$FF
        0x49, 0x0F, // EOR #$0F
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert_eq!(cpu.a, 0xF0);
}

#[test]
fn test_inx_iny() {
    let program = [
        0xA2, 0xFE, // LDX #$FE
        0xE8,       // INX
        0xE8,       // INX
        0xA0, 0x00, // LDY #$00
        0xC8,       // INY
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0xFF);
    assert!(!cpu.status.contains(StatusFlags::ZERO));
    assert!(cpu.status.contains(StatusFlags::NEGATIVE));
    
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0x00);
    assert!(cpu.status.contains(StatusFlags::ZERO));
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0x01);
}

#[test]
fn test_dex_dey() {
    let program = [
        0xA2, 0x01, // LDX #$01
        0xCA,       // DEX
        0xA0, 0x00, // LDY #$00
        0x88,       // DEY
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.x, 0x00);
    assert!(cpu.status.contains(StatusFlags::ZERO));
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    assert_eq!(cpu.y, 0xFF);
    assert!(cpu.status.contains(StatusFlags::NEGATIVE));
}

#[test]
fn test_cmp() {
    let program = [
        0xA9, 0x30, // LDA #$30
        0xC9, 0x30, // CMP #$30
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus);
    cpu.step(&mut bus);
    
    assert!(cpu.status.contains(StatusFlags::ZERO));
    assert!(cpu.status.contains(StatusFlags::CARRY)); // A >= M
}

#[test]
fn test_branch_taken() {
    let program = [
        0xA9, 0x00, // LDA #$00
        0xF0, 0x02, // BEQ +2
        0xA9, 0xFF, // LDA #$FF (skipped)
        0xA9, 0x42, // LDA #$42
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus); // LDA #$00
    cpu.step(&mut bus); // BEQ (taken)
    cpu.step(&mut bus); // LDA #$42
    
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn test_jsr_rts() {
    let program = [
        0x20, 0x06, 0x80, // JSR $8006
        0xA9, 0x42,       // LDA #$42
        0x00,             // BRK
        0xA9, 0x11,       // LDA #$11 (subroutine)
        0x60,             // RTS
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus); // JSR
    assert_eq!(cpu.pc, 0x8006);
    
    cpu.step(&mut bus); // LDA #$11
    assert_eq!(cpu.a, 0x11);
    
    cpu.step(&mut bus); // RTS
    assert_eq!(cpu.pc, 0x8003);
    
    cpu.step(&mut bus); // LDA #$42
    assert_eq!(cpu.a, 0x42);
}

#[test]
fn test_stack_operations() {
    let program = [
        0xA9, 0x42, // LDA #$42
        0x48,       // PHA
        0xA9, 0x00, // LDA #$00
        0x68,       // PLA
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    let initial_sp = cpu.sp;
    
    cpu.step(&mut bus); // LDA #$42
    cpu.step(&mut bus); // PHA
    assert_eq!(cpu.sp, initial_sp.wrapping_sub(1));
    
    cpu.step(&mut bus); // LDA #$00
    assert_eq!(cpu.a, 0x00);
    
    cpu.step(&mut bus); // PLA
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.sp, initial_sp);
}

#[test]
fn test_bit_operation() {
    let program = [
        0xA9, 0x80, // LDA #$80
        0x85, 0x10, // STA $10
        0xA9, 0xFF, // LDA #$FF
        0x24, 0x10, // BIT $10
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus); // LDA #$80
    cpu.step(&mut bus); // STA $10
    cpu.step(&mut bus); // LDA #$FF
    cpu.step(&mut bus); // BIT $10
    
    assert!(!cpu.status.contains(StatusFlags::ZERO)); // A & M != 0
    assert!(cpu.status.contains(StatusFlags::NEGATIVE)); // Bit 7 of M
    assert!(!cpu.status.contains(StatusFlags::OVERFLOW)); // Bit 6 of M
}

#[test]
fn test_rol_ror() {
    let program = [
        0xA9, 0x80, // LDA #$80
        0x2A,       // ROL A
        0x6A,       // ROR A
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus); // LDA #$80
    cpu.step(&mut bus); // ROL A
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(StatusFlags::CARRY));
    
    cpu.step(&mut bus); // ROR A
    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.status.contains(StatusFlags::CARRY));
}

#[test]
fn test_asl_lsr() {
    let program = [
        0xA9, 0x81, // LDA #$81
        0x0A,       // ASL A
        0x4A,       // LSR A
    ];
    let (mut cpu, mut bus) = setup_cpu(&program);
    
    cpu.step(&mut bus); // LDA #$81
    cpu.step(&mut bus); // ASL A
    assert_eq!(cpu.a, 0x02);
    assert!(cpu.status.contains(StatusFlags::CARRY));
    
    cpu.step(&mut bus); // LSR A
    assert_eq!(cpu.a, 0x01);
    assert!(!cpu.status.contains(StatusFlags::CARRY));
}

#[test]
fn test_addressing_modes() {
    let program = [
        // Zero page
        0xA9, 0x42, // LDA #$42
        0x85, 0x10, // STA $10
        0xA5, 0x10, // LDA $10
        
        // Zero page,X
        0xA2, 0x05, // LDX #$05
        0xB5, 0x0B, // LDA $0B,X (loads from $10)
        
        // Absolute
        0xAD, 0x00, 0x02, // LDA $0200
        
        // Indexed indirect (X)
        0xA9, 0x00, // LDA #$00
        0x85, 0x20, // STA $20
        0xA9, 0x02, // LDA #$02
        0x85, 0x21, // STA $21
        0xA1, 0x1B, // LDA ($1B,X) X=5, so ($20)
        
        // Indirect indexed (Y)
        0xA0, 0x03, // LDY #$03
        0xB1, 0x20, // LDA ($20),Y
    ];
    
    let (mut cpu, mut bus) = setup_cpu(&program);
    bus.write(0x0200, 0x55);
    bus.write(0x0203, 0x66);
    
    // Test zero page
    cpu.step(&mut bus); // LDA #$42
    cpu.step(&mut bus); // STA $10
    cpu.step(&mut bus); // LDA $10
    assert_eq!(cpu.a, 0x42);
    
    // Test zero page,X
    cpu.step(&mut bus); // LDX #$05
    cpu.step(&mut bus); // LDA $0B,X
    assert_eq!(cpu.a, 0x42);
    
    // Test absolute
    cpu.step(&mut bus); // LDA $0200
    assert_eq!(cpu.a, 0x55);
    
    // Test indexed indirect
    cpu.step(&mut bus); // LDA #$00
    cpu.step(&mut bus); // STA $20
    cpu.step(&mut bus); // LDA #$02
    cpu.step(&mut bus); // STA $21
    cpu.step(&mut bus); // LDA ($1B,X)
    assert_eq!(cpu.a, 0x55);
    
    // Test indirect indexed
    cpu.step(&mut bus); // LDY #$03
    cpu.step(&mut bus); // LDA ($20),Y
    assert_eq!(cpu.a, 0x66);
}

// Unofficial opcodes test removed - not implemented yet
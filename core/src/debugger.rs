use crate::{Cpu, Bus, Nes};
use crate::cpu::CpuBus;
use std::collections::{HashSet, HashMap};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BreakpointType {
    Execution,
    Read,
    Write,
}

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub address: u16,
    pub bp_type: BreakpointType,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerState {
    Running,
    Paused,
    StepInstruction,
    StepFrame,
}

pub struct Debugger {
    state: DebuggerState,
    breakpoints: HashMap<u16, Vec<Breakpoint>>,
    step_count: u32,
    watch_addresses: HashSet<u16>,
    trace_enabled: bool,
    trace_buffer: Vec<String>,
    last_pc: u16,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            state: DebuggerState::Running,
            breakpoints: HashMap::new(),
            step_count: 0,
            watch_addresses: HashSet::new(),
            trace_enabled: false,
            trace_buffer: Vec::new(),
            last_pc: 0,
        }
    }
    
    // Breakpoint management
    pub fn add_breakpoint(&mut self, address: u16, bp_type: BreakpointType) {
        let bp = Breakpoint {
            address,
            bp_type,
            enabled: true,
        };
        
        self.breakpoints
            .entry(address)
            .or_insert_with(Vec::new)
            .push(bp);
    }
    
    pub fn remove_breakpoint(&mut self, address: u16, bp_type: BreakpointType) {
        if let Some(bps) = self.breakpoints.get_mut(&address) {
            bps.retain(|bp| bp.bp_type != bp_type);
            if bps.is_empty() {
                self.breakpoints.remove(&address);
            }
        }
    }
    
    pub fn toggle_breakpoint(&mut self, address: u16, bp_type: BreakpointType) {
        if let Some(bps) = self.breakpoints.get_mut(&address) {
            if let Some(bp) = bps.iter_mut().find(|bp| bp.bp_type == bp_type) {
                bp.enabled = !bp.enabled;
                return;
            }
        }
        self.add_breakpoint(address, bp_type);
    }
    
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }
    
    // Execution control
    pub fn pause(&mut self) {
        self.state = DebuggerState::Paused;
    }
    
    pub fn resume(&mut self) {
        self.state = DebuggerState::Running;
    }
    
    pub fn step_instruction(&mut self) {
        self.state = DebuggerState::StepInstruction;
        self.step_count = 1;
    }
    
    pub fn step_frame(&mut self) {
        self.state = DebuggerState::StepFrame;
    }
    
    pub fn is_paused(&self) -> bool {
        matches!(self.state, DebuggerState::Paused)
    }
    
    // Watch addresses
    pub fn add_watch(&mut self, address: u16) {
        self.watch_addresses.insert(address);
    }
    
    pub fn remove_watch(&mut self, address: u16) {
        self.watch_addresses.remove(&address);
    }
    
    pub fn get_watches(&self) -> &HashSet<u16> {
        &self.watch_addresses
    }
    
    // Trace
    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
    }
    
    pub fn disable_trace(&mut self) {
        self.trace_enabled = false;
    }
    
    pub fn get_trace(&self) -> &[String] {
        &self.trace_buffer
    }
    
    pub fn clear_trace(&mut self) {
        self.trace_buffer.clear();
    }
    
    // Check if we should break
    pub fn check_breakpoint(&mut self, address: u16, bp_type: BreakpointType) -> bool {
        if let Some(bps) = self.breakpoints.get(&address) {
            for bp in bps {
                if bp.enabled && bp.bp_type == bp_type {
                    self.state = DebuggerState::Paused;
                    return true;
                }
            }
        }
        false
    }
    
    // Update debugger state after CPU step
    pub fn update_after_step(&mut self, cpu: &Cpu) {
        match self.state {
            DebuggerState::StepInstruction => {
                self.step_count -= 1;
                if self.step_count == 0 {
                    self.state = DebuggerState::Paused;
                }
            }
            DebuggerState::Running => {
                // Check execution breakpoint
                self.check_breakpoint(cpu.pc, BreakpointType::Execution);
            }
            _ => {}
        }
        
        // Trace execution if enabled
        if self.trace_enabled && cpu.pc != self.last_pc {
            let trace = format!(
                "PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} SP:{:02X} P:{:02X} CYC:{}",
                cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.status.bits(), cpu.cycles
            );
            self.trace_buffer.push(trace);
            
            // Limit trace buffer size
            if self.trace_buffer.len() > 10000 {
                self.trace_buffer.drain(0..5000);
            }
        }
        
        self.last_pc = cpu.pc;
    }
    
    // Update after frame for frame stepping
    pub fn update_after_frame(&mut self) {
        if self.state == DebuggerState::StepFrame {
            self.state = DebuggerState::Paused;
        }
    }
}

// Debug info display
pub struct DebugInfo<'a> {
    pub cpu: &'a Cpu,
    pub bus: &'a Bus,
}

impl<'a> fmt::Display for DebugInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== CPU State ===")?;
        writeln!(f, "PC: ${:04X}", self.cpu.pc)?;
        writeln!(f, "A:  ${:02X}", self.cpu.a)?;
        writeln!(f, "X:  ${:02X}", self.cpu.x)?;
        writeln!(f, "Y:  ${:02X}", self.cpu.y)?;
        writeln!(f, "SP: ${:02X}", self.cpu.sp)?;
        writeln!(f, "P:  {:08b} (NV-BDIZC)", self.cpu.status.bits())?;
        writeln!(f, "    {}{}{}{}{}{}{}{}", 
            if self.cpu.status.contains(crate::cpu::StatusFlags::NEGATIVE) { "N" } else { "-" },
            if self.cpu.status.contains(crate::cpu::StatusFlags::OVERFLOW) { "V" } else { "-" },
            "-",
            if self.cpu.status.contains(crate::cpu::StatusFlags::BREAK) { "B" } else { "-" },
            if self.cpu.status.contains(crate::cpu::StatusFlags::DECIMAL) { "D" } else { "-" },
            if self.cpu.status.contains(crate::cpu::StatusFlags::INTERRUPT) { "I" } else { "-" },
            if self.cpu.status.contains(crate::cpu::StatusFlags::ZERO) { "Z" } else { "-" },
            if self.cpu.status.contains(crate::cpu::StatusFlags::CARRY) { "C" } else { "-" }
        )?;
        writeln!(f, "Cycles: {}", self.cpu.cycles)?;
        Ok(())
    }
}

// Disassembler
pub fn disassemble(bus: &mut Bus, address: u16, count: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut addr = address;
    
    for _ in 0..count {
        let opcode = bus.read(addr);
        let (instruction, length) = decode_instruction(bus, addr);
        
        let mut bytes = format!("{:02X}", opcode);
        for i in 1..length {
            bytes.push_str(&format!(" {:02X}", bus.read(addr + i)));
        }
        
        result.push(format!("{:04X}: {:9} {}", addr, bytes, instruction));
        addr += length;
    }
    
    result
}

fn decode_instruction(bus: &mut Bus, addr: u16) -> (String, u16) {
    let opcode = bus.read(addr);
    
    // This is a simplified disassembler - in a real implementation,
    // you would decode all instructions properly
    match opcode {
        0x00 => ("BRK".to_string(), 1),
        0x01 => {
            let operand = bus.read(addr + 1);
            (format!("ORA (${:02X},X)", operand), 2)
        }
        0x05 => {
            let operand = bus.read(addr + 1);
            (format!("ORA ${:02X}", operand), 2)
        }
        0x06 => {
            let operand = bus.read(addr + 1);
            (format!("ASL ${:02X}", operand), 2)
        }
        0x08 => ("PHP".to_string(), 1),
        0x09 => {
            let operand = bus.read(addr + 1);
            (format!("ORA #${:02X}", operand), 2)
        }
        0x0A => ("ASL A".to_string(), 1),
        0x0D => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("ORA ${:04X}", operand), 3)
        }
        0x10 => {
            let offset = bus.read(addr + 1) as i8;
            let target = (addr as i32 + 2 + offset as i32) as u16;
            (format!("BPL ${:04X}", target), 2)
        }
        0x18 => ("CLC".to_string(), 1),
        0x20 => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("JSR ${:04X}", operand), 3)
        }
        0x28 => ("PLP".to_string(), 1),
        0x29 => {
            let operand = bus.read(addr + 1);
            (format!("AND #${:02X}", operand), 2)
        }
        0x2C => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("BIT ${:04X}", operand), 3)
        }
        0x30 => {
            let offset = bus.read(addr + 1) as i8;
            let target = (addr as i32 + 2 + offset as i32) as u16;
            (format!("BMI ${:04X}", target), 2)
        }
        0x38 => ("SEC".to_string(), 1),
        0x40 => ("RTI".to_string(), 1),
        0x48 => ("PHA".to_string(), 1),
        0x4C => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("JMP ${:04X}", operand), 3)
        }
        0x60 => ("RTS".to_string(), 1),
        0x68 => ("PLA".to_string(), 1),
        0x6C => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("JMP (${:04X})", operand), 3)
        }
        0x78 => ("SEI".to_string(), 1),
        0x88 => ("DEY".to_string(), 1),
        0x8D => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("STA ${:04X}", operand), 3)
        }
        0x9A => ("TXS".to_string(), 1),
        0xA0 => {
            let operand = bus.read(addr + 1);
            (format!("LDY #${:02X}", operand), 2)
        }
        0xA2 => {
            let operand = bus.read(addr + 1);
            (format!("LDX #${:02X}", operand), 2)
        }
        0xA9 => {
            let operand = bus.read(addr + 1);
            (format!("LDA #${:02X}", operand), 2)
        }
        0xAD => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("LDA ${:04X}", operand), 3)
        }
        0xBD => {
            let lo = bus.read(addr + 1);
            let hi = bus.read(addr + 2);
            let operand = (hi as u16) << 8 | lo as u16;
            (format!("LDA ${:04X},X", operand), 3)
        }
        0xC8 => ("INY".to_string(), 1),
        0xC9 => {
            let operand = bus.read(addr + 1);
            (format!("CMP #${:02X}", operand), 2)
        }
        0xCA => ("DEX".to_string(), 1),
        0xD0 => {
            let offset = bus.read(addr + 1) as i8;
            let target = (addr as i32 + 2 + offset as i32) as u16;
            (format!("BNE ${:04X}", target), 2)
        }
        0xE8 => ("INX".to_string(), 1),
        0xEA => ("NOP".to_string(), 1),
        0xF0 => {
            let offset = bus.read(addr + 1) as i8;
            let target = (addr as i32 + 2 + offset as i32) as u16;
            (format!("BEQ ${:04X}", target), 2)
        }
        _ => (format!(".db ${:02X}", opcode), 1),
    }
}

// Memory dump
pub fn dump_memory(bus: &mut Bus, start: u16, length: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut addr = start;
    
    for _ in 0..(length + 15) / 16 {
        let mut line = format!("{:04X}: ", addr);
        let mut ascii = String::new();
        
        for i in 0..16 {
            if (addr as usize + i) < (start as usize + length) {
                let byte = bus.read(addr + i as u16);
                line.push_str(&format!("{:02X} ", byte));
                
                if byte >= 0x20 && byte < 0x7F {
                    ascii.push(byte as char);
                } else {
                    ascii.push('.');
                }
            } else {
                line.push_str("   ");
            }
            
            if i == 7 {
                line.push(' ');
            }
        }
        
        line.push_str(" |");
        line.push_str(&ascii);
        line.push('|');
        
        result.push(line);
        addr += 16;
    }
    
    result
}

// Integration with Nes
impl Nes {
    pub fn attach_debugger(&mut self) -> &mut Debugger {
        // This would require adding a debugger field to Nes struct
        // For now, we'll just show how it would be used
        unimplemented!("Debugger integration requires modifying Nes struct")
    }
}
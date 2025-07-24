use ccnes_core::{Debugger, DebuggerState, BreakpointType, DebugInfo};
use ccnes_core::{Nes, debugger};
use sdl2::keyboard::Keycode;
use std::io::{self, Write};

pub struct DebuggerUI {
    debugger: Debugger,
    show_debugger: bool,
    command_buffer: String,
}

impl DebuggerUI {
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
            show_debugger: false,
            command_buffer: String::new(),
        }
    }
    
    pub fn toggle(&mut self) {
        self.show_debugger = !self.show_debugger;
        if self.show_debugger {
            self.debugger.pause();
            println!("\n=== Debugger Enabled ===");
            self.print_help();
        } else {
            self.debugger.resume();
            println!("\n=== Debugger Disabled ===");
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.show_debugger
    }
    
    pub fn get_debugger(&mut self) -> &mut Debugger {
        &mut self.debugger
    }
    
    pub fn handle_key(&mut self, keycode: Keycode, nes: &mut Nes) -> bool {
        if !self.show_debugger {
            return false;
        }
        
        match keycode {
            Keycode::F1 => {
                self.print_help();
                true
            }
            Keycode::F2 => {
                self.print_cpu_state(nes);
                true
            }
            Keycode::F3 => {
                self.debugger.step_instruction();
                println!("Stepping one instruction...");
                true
            }
            Keycode::F4 => {
                self.debugger.resume();
                println!("Resuming execution...");
                true
            }
            Keycode::F5 => {
                self.debugger.pause();
                println!("Paused execution");
                true
            }
            _ => false,
        }
    }
    
    pub fn update(&mut self, nes: &mut Nes) {
        self.debugger.update_after_step(&nes.cpu);
        
        if self.debugger.is_paused() && self.show_debugger {
            self.print_status(nes);
        }
    }
    
    pub fn update_frame(&mut self) {
        self.debugger.update_after_frame();
    }
    
    fn print_help(&self) {
        println!("\nDebugger Commands:");
        println!("  F1  - Show this help");
        println!("  F2  - Show CPU state");
        println!("  F3  - Step one instruction");
        println!("  F4  - Resume execution");
        println!("  F5  - Pause execution");
        println!("  F10 - Toggle debugger");
        println!();
    }
    
    fn print_cpu_state(&self, nes: &Nes) {
        let debug_info = DebugInfo {
            cpu: &nes.cpu,
            bus: &nes.bus,
        };
        println!("{}", debug_info);
    }
    
    fn print_status(&self, nes: &mut Nes) {
        println!("\n--- Debugger Paused ---");
        
        // Show current instruction
        let pc = nes.cpu.pc;
        let disasm = debugger::disassemble(&mut nes.bus, pc, 1);
        if let Some(instruction) = disasm.first() {
            println!("Next: {}", instruction);
        }
        
        // Show nearby instructions
        let start = pc.saturating_sub(3);
        let disasm = debugger::disassemble(&mut nes.bus, start, 7);
        println!("\nNearby instructions:");
        for line in disasm {
            let addr = u16::from_str_radix(&line[0..4], 16).unwrap_or(0);
            if addr == pc {
                println!("> {}", line);
            } else {
                println!("  {}", line);
            }
        }
    }
    
    pub fn process_command(&mut self, nes: &mut Nes) {
        if !self.show_debugger {
            return;
        }
        
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            if parts.is_empty() {
                return;
            }
            
            match parts[0] {
                "help" | "h" => self.print_help(),
                "step" | "s" => {
                    let count = parts.get(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                    for _ in 0..count {
                        self.debugger.step_instruction();
                    }
                }
                "continue" | "c" => self.debugger.resume(),
                "break" | "b" => {
                    if let Some(addr_str) = parts.get(1) {
                        if let Ok(addr) = u16::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                            self.debugger.add_breakpoint(addr, BreakpointType::Execution);
                            println!("Breakpoint set at ${:04X}", addr);
                        }
                    }
                }
                "disasm" | "d" => {
                    let addr = parts.get(1)
                        .and_then(|s| u16::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                        .unwrap_or(nes.cpu.pc);
                    let count = parts.get(2)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10);
                    
                    let disasm = debugger::disassemble(&mut nes.bus, addr, count);
                    for line in disasm {
                        println!("{}", line);
                    }
                }
                "mem" | "m" => {
                    if let Some(addr_str) = parts.get(1) {
                        if let Ok(addr) = u16::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                            let length = parts.get(2)
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(128);
                            
                            let dump = debugger::dump_memory(&mut nes.bus, addr, length);
                            for line in dump {
                                println!("{}", line);
                            }
                        }
                    }
                }
                "trace" => {
                    if parts.get(1).map(|&s| s == "on").unwrap_or(false) {
                        self.debugger.enable_trace();
                        println!("Trace enabled");
                    } else if parts.get(1).map(|&s| s == "off").unwrap_or(false) {
                        self.debugger.disable_trace();
                        println!("Trace disabled");
                    } else {
                        let trace = self.debugger.get_trace();
                        let start = trace.len().saturating_sub(20);
                        for line in &trace[start..] {
                            println!("{}", line);
                        }
                    }
                }
                _ => println!("Unknown command: {}", parts[0]),
            }
        }
    }
}
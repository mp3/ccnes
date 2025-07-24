use crate::{Cpu, Ppu, Apu, Bus, Cartridge, Clock};

pub struct Nes {
    cpu: Cpu,
    bus: Bus,
    clock: Clock,
}

impl Nes {
    pub fn new() -> Self {
        let ppu = Ppu::new();
        let apu = Apu::new();
        let bus = Bus::new(ppu, apu);
        let cpu = Cpu::new();
        
        Self {
            cpu,
            bus,
            clock: Clock {
                cpu_cycles: 0,
                ppu_cycles: 0,
                apu_cycles: 0,
            },
        }
    }
    
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.load_cartridge(cartridge);
        self.reset();
    }
    
    pub fn reset(&mut self) {
        self.cpu.reset(&self.bus);
        self.clock = Clock {
            cpu_cycles: 0,
            ppu_cycles: 0,
            apu_cycles: 0,
        };
    }
    
    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.step(&mut self.bus);
        self.clock.cpu_cycles += cpu_cycles as u64;
        
        // PPU runs at 3x CPU speed
        for _ in 0..(cpu_cycles * 3) {
            // In a full implementation, we'd step the PPU here
            self.clock.ppu_cycles += 1;
        }
        
        // APU runs at CPU speed
        for _ in 0..cpu_cycles {
            // In a full implementation, we'd step the APU here
            self.clock.apu_cycles += 1;
        }
    }
    
    pub fn run_frame(&mut self) {
        // Run until we've completed a frame (roughly 29780 CPU cycles)
        let target_cycles = self.clock.cpu_cycles + 29780;
        while self.clock.cpu_cycles < target_cycles {
            self.step();
        }
    }
    
    pub fn set_controller1(&mut self, state: u8) {
        self.bus.set_controller1(state);
    }
    
    pub fn set_controller2(&mut self, state: u8) {
        self.bus.set_controller2(state);
    }
}
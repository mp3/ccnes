use crate::{Cpu, Ppu, Apu, Bus, Cartridge, Clock, Controller};

pub struct Nes {
    pub cpu: Cpu,
    pub bus: Bus,
    pub clock: Clock,
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
        self.cpu.reset(&mut self.bus);
        self.clock = Clock {
            cpu_cycles: 0,
            ppu_cycles: 0,
            apu_cycles: 0,
        };
    }
    
    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.step(&mut self.bus);
        self.clock.cpu_cycles += cpu_cycles as u64;
        
        // Bus tick handles PPU and APU timing
        for _ in 0..cpu_cycles {
            self.bus.tick(&mut self.cpu);
            self.clock.ppu_cycles += 3;
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
    
    pub fn get_framebuffer(&self) -> &[u32] {
        &self.bus.ppu.framebuffer
    }
    
    pub fn set_controller1(&mut self, state: u8) {
        self.bus.set_controller1(state);
    }
    
    pub fn set_controller2(&mut self, state: u8) {
        self.bus.set_controller2(state);
    }
    
    pub fn set_controller1_from_controller(&mut self, controller: &Controller) {
        self.bus.set_controller1(controller.get_state());
    }
    
    pub fn set_controller2_from_controller(&mut self, controller: &Controller) {
        self.bus.set_controller2(controller.get_state());
    }
}
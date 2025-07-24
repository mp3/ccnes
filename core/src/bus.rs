use crate::cpu::{Cpu, CpuBus};
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::cartridge::Cartridge;

pub struct Bus {
    ram: [u8; 0x800],      // 2KB internal RAM
    pub ppu: Ppu,
    pub apu: Apu,
    pub cartridge: Option<Cartridge>,
    controller1: u8,
    controller2: u8,
    controller1_state: u8,
    controller2_state: u8,
    controller_strobe: bool,
}

impl Bus {
    pub fn new(ppu: Ppu, apu: Apu) -> Self {
        Self {
            ram: [0; 0x800],
            ppu,
            apu,
            cartridge: None,
            controller1: 0,
            controller2: 0,
            controller1_state: 0,
            controller2_state: 0,
            controller_strobe: false,
        }
    }
    
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }
    
    pub fn set_controller1(&mut self, state: u8) {
        self.controller1_state = state;
    }
    
    pub fn set_controller2(&mut self, state: u8) {
        self.controller2_state = state;
    }
    
    pub fn tick(&mut self, cpu: &mut Cpu) {
        // PPU runs 3 times per CPU cycle
        let mut nmi = false;
        for _ in 0..3 {
            if let Some(ref cartridge) = self.cartridge {
                if self.ppu.step(cartridge) {
                    nmi = true;
                }
            }
        }
        
        if nmi {
            cpu.trigger_nmi();
        }
        
        // APU runs once per CPU cycle
        self.apu.step();
    }
}

impl CpuBus for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                // RAM and mirrors
                self.ram[(addr & 0x7FF) as usize]
            }
            0x2000..=0x3FFF => {
                // PPU registers and mirrors
                self.ppu.read_register((addr & 0x7) as u8)
            }
            0x4000..=0x4015 => {
                // APU registers
                self.apu.read_register(addr)
            }
            0x4016 => {
                // Controller 1
                if self.controller_strobe {
                    self.controller1 = self.controller1_state;
                }
                let bit = self.controller1 & 0x01;
                self.controller1 >>= 1;
                self.controller1 |= 0x80;
                bit
            }
            0x4017 => {
                // Controller 2
                if self.controller_strobe {
                    self.controller2 = self.controller2_state;
                }
                let bit = self.controller2 & 0x01;
                self.controller2 >>= 1;
                self.controller2 |= 0x80;
                bit
            }
            0x4018..=0x401F => {
                // APU and I/O functionality that is normally disabled
                0
            }
            0x4020..=0xFFFF => {
                // Cartridge space
                if let Some(ref cart) = self.cartridge {
                    cart.read_prg(addr)
                } else {
                    0
                }
            }
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                // RAM and mirrors
                self.ram[(addr & 0x7FF) as usize] = value;
            }
            0x2000..=0x3FFF => {
                // PPU registers and mirrors
                self.ppu.write_register((addr & 0x7) as u8, value);
            }
            0x4000..=0x4015 => {
                // APU registers
                self.apu.write_register(addr, value);
            }
            0x4016 => {
                // Controller strobe
                self.controller_strobe = value & 0x01 != 0;
                if self.controller_strobe {
                    self.controller1 = self.controller1_state;
                    self.controller2 = self.controller2_state;
                }
            }
            0x4017 => {
                // APU Frame Counter
                self.apu.write_register(addr, value);
            }
            0x4018..=0x401F => {
                // APU and I/O functionality that is normally disabled
            }
            0x4020..=0xFFFF => {
                // Cartridge space
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_prg(addr, value);
                }
            }
        }
    }
}
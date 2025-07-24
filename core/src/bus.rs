use crate::cpu::CpuBus;
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::cartridge::Cartridge;

pub struct Bus {
    ram: [u8; 0x800],      // 2KB internal RAM
    ppu: Ppu,
    apu: Apu,
    cartridge: Option<Cartridge>,
    controller1: u8,
    controller2: u8,
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
        }
    }
    
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }
    
    pub fn set_controller1(&mut self, state: u8) {
        self.controller1 = state;
    }
    
    pub fn set_controller2(&mut self, state: u8) {
        self.controller2 = state;
    }
}

impl CpuBus for Bus {
    fn read(&self, addr: u16) -> u8 {
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
                self.controller1
            }
            0x4017 => {
                // Controller 2
                self.controller2
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
                // Controller 1
                // Writing here triggers controller polling
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
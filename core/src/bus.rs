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
    oam_dma_page: Option<u8>,
    oam_dma_cycle: u16,
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
            oam_dma_page: None,
            oam_dma_cycle: 0,
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
        // Handle OAM DMA if active
        if let Some(page) = self.oam_dma_page {
            // DMA takes 513 or 514 cycles
            if self.oam_dma_cycle < 512 {
                if self.oam_dma_cycle % 2 == 0 {
                    // Read cycle
                    let addr = ((page as u16) << 8) | ((self.oam_dma_cycle / 2) as u16);
                    let data = self.read(addr);
                    self.ppu.write_oam_byte((self.oam_dma_cycle / 2) as u8, data);
                }
                self.oam_dma_cycle += 1;
            } else {
                // DMA complete
                self.oam_dma_page = None;
                self.oam_dma_cycle = 0;
            }
            // CPU is stalled during DMA
            cpu.stall(1);
        }
        
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
            0x4000..=0x4013 => {
                // APU registers
                self.apu.write_register(addr, value);
            }
            0x4014 => {
                // OAM DMA
                self.oam_dma_page = Some(value);
                self.oam_dma_cycle = 0;
            }
            0x4015 => {
                // APU control
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
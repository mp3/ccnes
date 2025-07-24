#[derive(Debug, Clone)]
pub struct Apu {
    // Pulse channels
    pulse1: PulseChannel,
    pulse2: PulseChannel,
    
    // Triangle channel
    triangle: TriangleChannel,
    
    // Noise channel
    noise: NoiseChannel,
    
    // DMC channel
    dmc: DmcChannel,
    
    // Frame counter
    frame_counter: u8,
    frame_mode: bool,
    frame_irq: bool,
    
    cycles: u32,
}

#[derive(Debug, Clone, Default)]
struct PulseChannel {
    enabled: bool,
    duty: u8,
    length_counter: u8,
    timer: u16,
    volume: u8,
}

#[derive(Debug, Clone, Default)]
struct TriangleChannel {
    enabled: bool,
    length_counter: u8,
    timer: u16,
    linear_counter: u8,
}

#[derive(Debug, Clone, Default)]
struct NoiseChannel {
    enabled: bool,
    length_counter: u8,
    timer: u16,
    volume: u8,
    mode: bool,
}

#[derive(Debug, Clone, Default)]
struct DmcChannel {
    enabled: bool,
    rate: u8,
    length: u16,
    address: u16,
    sample: u8,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: PulseChannel::default(),
            pulse2: PulseChannel::default(),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::default(),
            dmc: DmcChannel::default(),
            frame_counter: 0,
            frame_mode: false,
            frame_irq: false,
            cycles: 0,
        }
    }
    
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                // Status register
                let mut status = 0;
                if self.pulse1.length_counter > 0 {
                    status |= 0x01;
                }
                if self.pulse2.length_counter > 0 {
                    status |= 0x02;
                }
                if self.triangle.length_counter > 0 {
                    status |= 0x04;
                }
                if self.noise.length_counter > 0 {
                    status |= 0x08;
                }
                if self.dmc.length > 0 {
                    status |= 0x10;
                }
                if self.frame_irq {
                    status |= 0x40;
                }
                status
            }
            _ => 0,
        }
    }
    
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000..=0x4003 => {
                // Pulse 1
                Self::write_pulse(&mut self.pulse1, addr & 0x3, value);
            }
            0x4004..=0x4007 => {
                // Pulse 2
                Self::write_pulse(&mut self.pulse2, addr & 0x3, value);
            }
            0x4008..=0x400B => {
                // Triangle
                self.write_triangle(addr & 0x3, value);
            }
            0x400C..=0x400F => {
                // Noise
                self.write_noise(addr & 0x3, value);
            }
            0x4010..=0x4013 => {
                // DMC
                self.write_dmc(addr & 0x3, value);
            }
            0x4015 => {
                // Control
                self.pulse1.enabled = (value & 0x01) != 0;
                self.pulse2.enabled = (value & 0x02) != 0;
                self.triangle.enabled = (value & 0x04) != 0;
                self.noise.enabled = (value & 0x08) != 0;
                self.dmc.enabled = (value & 0x10) != 0;
                
                if !self.pulse1.enabled {
                    self.pulse1.length_counter = 0;
                }
                if !self.pulse2.enabled {
                    self.pulse2.length_counter = 0;
                }
                if !self.triangle.enabled {
                    self.triangle.length_counter = 0;
                }
                if !self.noise.enabled {
                    self.noise.length_counter = 0;
                }
            }
            0x4017 => {
                // Frame counter
                self.frame_mode = (value & 0x80) != 0;
                self.frame_irq = (value & 0x40) == 0;
            }
            _ => {}
        }
    }
    
    fn write_pulse(channel: &mut PulseChannel, reg: u16, value: u8) {
        match reg {
            0 => {
                channel.duty = (value >> 6) & 0x3;
                channel.volume = value & 0xF;
            }
            1 => {
                // Sweep unit (not implemented in this basic version)
            }
            2 => {
                channel.timer = (channel.timer & 0xFF00) | value as u16;
            }
            3 => {
                channel.timer = (channel.timer & 0x00FF) | ((value as u16 & 0x7) << 8);
                channel.length_counter = value >> 3;
            }
            _ => {}
        }
    }
    
    fn write_triangle(&mut self, reg: u16, value: u8) {
        match reg {
            0 => {
                self.triangle.linear_counter = value & 0x7F;
            }
            2 => {
                self.triangle.timer = (self.triangle.timer & 0xFF00) | value as u16;
            }
            3 => {
                self.triangle.timer = (self.triangle.timer & 0x00FF) | ((value as u16 & 0x7) << 8);
                self.triangle.length_counter = value >> 3;
            }
            _ => {}
        }
    }
    
    fn write_noise(&mut self, reg: u16, value: u8) {
        match reg {
            0 => {
                self.noise.volume = value & 0xF;
            }
            2 => {
                self.noise.mode = (value & 0x80) != 0;
                self.noise.timer = value as u16 & 0xF;
            }
            3 => {
                self.noise.length_counter = value >> 3;
            }
            _ => {}
        }
    }
    
    fn write_dmc(&mut self, reg: u16, value: u8) {
        match reg {
            0 => {
                self.dmc.rate = value & 0xF;
            }
            1 => {
                self.dmc.sample = value & 0x7F;
            }
            2 => {
                self.dmc.address = 0xC000 | ((value as u16) << 6);
            }
            3 => {
                self.dmc.length = ((value as u16) << 4) | 1;
            }
            _ => {}
        }
    }
    
    pub fn step(&mut self) {
        self.cycles += 1;
        
        // Frame counter logic would go here
        // This is a simplified version
    }
}
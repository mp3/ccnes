mod filters;
mod resampler;
mod buffer;

use filters::NesAudioFilter;
use resampler::Resampler;
use buffer::AdaptiveBuffer;

pub use resampler::ResamplerQuality;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

const DMC_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

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
    frame_irq_inhibit: bool,
    
    cycles: u32,
    frame_cycles: u32,
    
    // Audio output
    sample_rate: u32,
    sample_counter: f32,
    samples: Vec<f32>,
    
    // Audio processing
    filter: NesAudioFilter,
    resampler: Resampler,
    output_buffer: AdaptiveBuffer,
}

#[derive(Debug, Clone, Default)]
struct PulseChannel {
    enabled: bool,
    duty: u8,
    length_counter: u8,
    timer: u16,
    timer_period: u16,
    volume: u8,
    constant_volume: bool,
    envelope: u8,
    envelope_start: bool,
    envelope_period: u8,
    envelope_value: u8,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    duty_position: u8,
}

#[derive(Debug, Clone, Default)]
struct TriangleChannel {
    enabled: bool,
    length_counter: u8,
    timer: u16,
    timer_period: u16,
    linear_counter: u8,
    linear_counter_reload: u8,
    linear_counter_reload_flag: bool,
    control_flag: bool,
    sequence_position: u8,
}

#[derive(Debug, Clone, Default)]
struct NoiseChannel {
    enabled: bool,
    length_counter: u8,
    timer: u16,
    timer_period: u16,
    volume: u8,
    constant_volume: bool,
    envelope: u8,
    envelope_start: bool,
    envelope_period: u8,
    envelope_value: u8,
    mode: bool,
    shift_register: u16,
}

#[derive(Debug, Clone)]
struct DmcChannel {
    enabled: bool,
    
    // Frequency
    rate: u8,
    timer: u16,
    timer_period: u16,
    
    // Memory reader
    sample_address: u16,
    sample_length: u16,
    current_address: u16,
    bytes_remaining: u16,
    
    // Sample buffer
    sample_buffer: Option<u8>,
    
    // Output unit
    shift_register: u8,
    bits_remaining: u8,
    output_level: u8,
    silence: bool,
    
    // Flags
    loop_flag: bool,
    irq_enabled: bool,
    interrupt: bool,
}

impl Default for DmcChannel {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: 0,
            timer: 0,
            timer_period: 0,
            sample_address: 0xC000,
            sample_length: 0,
            current_address: 0xC000,
            bytes_remaining: 0,
            sample_buffer: None,
            shift_register: 0,
            bits_remaining: 0,
            output_level: 0,
            silence: true,
            loop_flag: false,
            irq_enabled: false,
            interrupt: false,
        }
    }
}

impl Apu {
    pub fn new() -> Self {
        Self::with_sample_rate(44100)
    }
    
    pub fn with_sample_rate(sample_rate: u32) -> Self {
        let mut noise = NoiseChannel::default();
        noise.shift_register = 1;
        
        let cpu_rate = 1789773.0; // NTSC CPU frequency
        
        Self {
            pulse1: PulseChannel::default(),
            pulse2: PulseChannel::default(),
            triangle: TriangleChannel::default(),
            noise,
            dmc: DmcChannel::default(),
            frame_counter: 0,
            frame_mode: false,
            frame_irq: false,
            frame_irq_inhibit: false,
            cycles: 0,
            frame_cycles: 0,
            sample_rate,
            sample_counter: 0.0,
            samples: Vec::new(),
            filter: NesAudioFilter::new(sample_rate as f32),
            resampler: Resampler::new(ResamplerQuality::Medium, cpu_rate, sample_rate as f32),
            output_buffer: AdaptiveBuffer::new(sample_rate as f32, 20.0), // 20ms latency target
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
                if self.dmc.bytes_remaining > 0 {
                    status |= 0x10;
                }
                if self.dmc.interrupt {
                    status |= 0x80;
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
                
                if self.dmc.enabled && self.dmc.bytes_remaining == 0 {
                    // Restart DMC
                    self.dmc.current_address = self.dmc.sample_address;
                    self.dmc.bytes_remaining = self.dmc.sample_length;
                } else if !self.dmc.enabled {
                    self.dmc.bytes_remaining = 0;
                }
                
                // Clear DMC interrupt
                self.dmc.interrupt = false;
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
                channel.envelope_period = value & 0xF;
                channel.constant_volume = (value & 0x10) != 0;
                channel.volume = value & 0xF;
            }
            1 => {
                // Sweep unit
                channel.sweep_enabled = (value & 0x80) != 0;
                channel.sweep_period = (value >> 4) & 0x7;
                channel.sweep_negate = (value & 0x08) != 0;
                channel.sweep_shift = value & 0x7;
                channel.sweep_reload = true;
            }
            2 => {
                channel.timer_period = (channel.timer_period & 0xFF00) | value as u16;
            }
            3 => {
                channel.timer_period = (channel.timer_period & 0x00FF) | ((value as u16 & 0x7) << 8);
                channel.timer = channel.timer_period;
                channel.length_counter = LENGTH_TABLE[(value >> 3) as usize];
                channel.envelope_start = true;
                channel.duty_position = 0;
            }
            _ => {}
        }
    }
    
    fn write_triangle(&mut self, reg: u16, value: u8) {
        match reg {
            0 => {
                self.triangle.control_flag = (value & 0x80) != 0;
                self.triangle.linear_counter_reload = value & 0x7F;
            }
            2 => {
                self.triangle.timer_period = (self.triangle.timer_period & 0xFF00) | value as u16;
            }
            3 => {
                self.triangle.timer_period = (self.triangle.timer_period & 0x00FF) | ((value as u16 & 0x7) << 8);
                self.triangle.timer = self.triangle.timer_period;
                self.triangle.length_counter = LENGTH_TABLE[(value >> 3) as usize];
                self.triangle.linear_counter_reload_flag = true;
            }
            _ => {}
        }
    }
    
    fn write_noise(&mut self, reg: u16, value: u8) {
        match reg {
            0 => {
                self.noise.envelope_period = value & 0xF;
                self.noise.constant_volume = (value & 0x10) != 0;
                self.noise.volume = value & 0xF;
            }
            2 => {
                self.noise.mode = (value & 0x80) != 0;
                self.noise.timer_period = NOISE_PERIOD_TABLE[(value & 0xF) as usize];
            }
            3 => {
                self.noise.length_counter = LENGTH_TABLE[(value >> 3) as usize];
                self.noise.envelope_start = true;
            }
            _ => {}
        }
    }
    
    fn write_dmc(&mut self, reg: u16, value: u8) {
        match reg {
            0 => {
                // $4010: Flags and rate
                self.dmc.irq_enabled = (value & 0x80) != 0;
                self.dmc.loop_flag = (value & 0x40) != 0;
                self.dmc.rate = value & 0x0F;
                self.dmc.timer_period = DMC_RATE_TABLE[self.dmc.rate as usize];
                
                if !self.dmc.irq_enabled {
                    self.dmc.interrupt = false;
                }
            }
            1 => {
                // $4011: Direct load
                self.dmc.output_level = value & 0x7F;
            }
            2 => {
                // $4012: Sample address
                self.dmc.sample_address = 0xC000 | ((value as u16) << 6);
            }
            3 => {
                // $4013: Sample length
                self.dmc.sample_length = ((value as u16) << 4) | 1;
            }
            _ => {}
        }
    }
    
    pub fn step(&mut self) {
        self.cycles += 1;
        
        // Clock timers
        self.clock_timers();
        
        // Frame counter
        self.frame_cycles += 1;
        
        if !self.frame_mode {
            // 4-step sequence (60 Hz)
            match self.frame_cycles {
                7457 => self.clock_quarter_frame(),
                14913 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                22371 => self.clock_quarter_frame(),
                29829 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    if !self.frame_irq_inhibit {
                        self.frame_irq = true;
                    }
                    self.frame_cycles = 0;
                }
                _ => {}
            }
        } else {
            // 5-step sequence (48 Hz)
            match self.frame_cycles {
                7457 => self.clock_quarter_frame(),
                14913 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                22371 => self.clock_quarter_frame(),
                29829 => {
                    // Do nothing
                }
                37281 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    self.frame_cycles = 0;
                }
                _ => {}
            }
        }
        
        // Generate sample
        self.generate_sample();
    }
    
    fn clock_timers(&mut self) {
        // Clock pulse channels every other CPU cycle
        if self.cycles % 2 == 0 {
            Self::clock_pulse(&mut self.pulse1);
            Self::clock_pulse(&mut self.pulse2);
            self.clock_noise();
        }
        
        // Clock triangle every CPU cycle
        self.clock_triangle();
        
        // Clock DMC
        self.clock_dmc();
    }
    
    fn clock_pulse(channel: &mut PulseChannel) {
        if channel.timer > 0 {
            channel.timer -= 1;
        } else {
            channel.timer = channel.timer_period;
            channel.duty_position = (channel.duty_position + 1) % 8;
        }
    }
    
    fn clock_triangle(&mut self) {
        if self.triangle.linear_counter > 0 && self.triangle.length_counter > 0 {
            if self.triangle.timer > 0 {
                self.triangle.timer -= 1;
            } else {
                self.triangle.timer = self.triangle.timer_period;
                if self.triangle.length_counter > 0 && self.triangle.linear_counter > 0 {
                    self.triangle.sequence_position = (self.triangle.sequence_position + 1) % 32;
                }
            }
        }
    }
    
    fn clock_noise(&mut self) {
        if self.noise.timer > 0 {
            self.noise.timer -= 1;
        } else {
            self.noise.timer = self.noise.timer_period;
            
            let feedback = if self.noise.mode {
                // Mode 1: feedback from bits 0 and 6
                ((self.noise.shift_register & 1) ^ ((self.noise.shift_register >> 6) & 1)) != 0
            } else {
                // Mode 0: feedback from bits 0 and 1
                ((self.noise.shift_register & 1) ^ ((self.noise.shift_register >> 1) & 1)) != 0
            };
            
            self.noise.shift_register >>= 1;
            if feedback {
                self.noise.shift_register |= 0x4000;
            }
        }
    }
    
    fn clock_dmc(&mut self) {
        // Clock output unit
        if self.dmc.timer > 0 {
            self.dmc.timer -= 1;
        } else {
            self.dmc.timer = self.dmc.timer_period;
            
            if !self.dmc.silence {
                if (self.dmc.shift_register & 1) != 0 {
                    // Increase level
                    if self.dmc.output_level <= 125 {
                        self.dmc.output_level += 2;
                    }
                } else {
                    // Decrease level
                    if self.dmc.output_level >= 2 {
                        self.dmc.output_level -= 2;
                    }
                }
            }
            
            self.dmc.shift_register >>= 1;
            self.dmc.bits_remaining = self.dmc.bits_remaining.saturating_sub(1);
            
            if self.dmc.bits_remaining == 0 {
                self.dmc.bits_remaining = 8;
                
                if let Some(sample) = self.dmc.sample_buffer {
                    self.dmc.shift_register = sample;
                    self.dmc.sample_buffer = None;
                    self.dmc.silence = false;
                } else {
                    self.dmc.silence = true;
                }
            }
        }
        
        // Check if we need to fetch a new sample
        if self.dmc.sample_buffer.is_none() && self.dmc.bytes_remaining > 0 {
            // In a real implementation, this would trigger a DMA read
            // For now, we'll simulate it with a pseudo-random value based on address
            // This provides more realistic audio than a constant value
            let sample_value = ((self.dmc.current_address >> 1) ^ (self.dmc.current_address >> 3)) as u8;
            self.dmc.sample_buffer = Some(sample_value);
            
            self.dmc.current_address = self.dmc.current_address.wrapping_add(1);
            if self.dmc.current_address == 0 {
                self.dmc.current_address = 0x8000;
            }
            self.dmc.bytes_remaining -= 1;
            
            if self.dmc.bytes_remaining == 0 {
                if self.dmc.loop_flag {
                    self.dmc.current_address = self.dmc.sample_address;
                    self.dmc.bytes_remaining = self.dmc.sample_length;
                } else if self.dmc.irq_enabled {
                    self.dmc.interrupt = true;
                }
            }
        }
    }
    
    fn clock_quarter_frame(&mut self) {
        // Clock envelopes
        Self::clock_envelope_pulse(&mut self.pulse1);
        Self::clock_envelope_pulse(&mut self.pulse2);
        Self::clock_envelope_noise(&mut self.noise);
        
        // Clock triangle linear counter
        if self.triangle.linear_counter_reload_flag {
            self.triangle.linear_counter = self.triangle.linear_counter_reload;
        } else if self.triangle.linear_counter > 0 {
            self.triangle.linear_counter -= 1;
        }
        
        if !self.triangle.control_flag {
            self.triangle.linear_counter_reload_flag = false;
        }
    }
    
    fn clock_half_frame(&mut self) {
        // Clock length counters
        Self::clock_length_counter(&mut self.pulse1.length_counter);
        Self::clock_length_counter(&mut self.pulse2.length_counter);
        Self::clock_length_counter(&mut self.triangle.length_counter);
        Self::clock_length_counter(&mut self.noise.length_counter);
        
        // Clock sweep units  
        let mut pulse1 = self.pulse1.clone();
        let mut pulse2 = self.pulse2.clone();
        self.clock_sweep(&mut pulse1, false);
        self.clock_sweep(&mut pulse2, true);
        self.pulse1 = pulse1;
        self.pulse2 = pulse2;
    }
    
    fn clock_envelope_pulse(channel: &mut PulseChannel) {
        if channel.envelope_start {
            channel.envelope_value = 15;
            channel.envelope = channel.envelope_period;
            channel.envelope_start = false;
        } else if channel.envelope > 0 {
            channel.envelope -= 1;
        } else {
            if channel.envelope_value > 0 {
                channel.envelope_value -= 1;
            } else if channel.envelope_period > 0 {
                channel.envelope_value = 15;
            }
            channel.envelope = channel.envelope_period;
        }
    }
    
    fn clock_envelope_noise(channel: &mut NoiseChannel) {
        if channel.envelope_start {
            channel.envelope_value = 15;
            channel.envelope = channel.envelope_period;
            channel.envelope_start = false;
        } else if channel.envelope > 0 {
            channel.envelope -= 1;
        } else {
            if channel.envelope_value > 0 {
                channel.envelope_value -= 1;
            } else if channel.envelope_period > 0 {
                channel.envelope_value = 15;
            }
            channel.envelope = channel.envelope_period;
        }
    }
    
    fn clock_length_counter(counter: &mut u8) {
        if *counter > 0 {
            *counter -= 1;
        }
    }
    
    fn clock_sweep(&mut self, channel: &mut PulseChannel, second_channel: bool) {
        // Note: In a real implementation, the divider would be stored in the channel
        // For now, we'll simplify and just use the period directly
        
        if channel.sweep_reload {
            channel.sweep_reload = false;
            if channel.sweep_enabled {
                self.sweep_target(channel, second_channel);
            }
        }
    }
    
    fn sweep_target(&mut self, channel: &mut PulseChannel, second_channel: bool) {
        let period = channel.timer_period;
        let mut change = period >> channel.sweep_shift;
        
        if channel.sweep_negate {
            if second_channel {
                change = (!change).wrapping_add(1);
            } else {
                change = !change;
            }
        }
        
        let target = period.wrapping_add(change);
        if target < 0x800 && channel.timer_period >= 8 {
            channel.timer_period = target;
        }
    }
    
    fn generate_sample(&mut self) {
        // Sample rate conversion
        let cpu_rate = 1789773.0; // NTSC CPU frequency
        let sample_period = cpu_rate / self.sample_rate as f32;
        
        self.sample_counter += 1.0;
        if self.sample_counter >= sample_period {
            self.sample_counter -= sample_period;
            
            // Mix channels
            let pulse1 = self.get_pulse_output(&self.pulse1);
            let pulse2 = self.get_pulse_output(&self.pulse2);
            let triangle = self.get_triangle_output();
            let noise = self.get_noise_output();
            let dmc = self.dmc.output_level as f32;
            
            // Improved non-linear mixing with better approximation
            let pulse_out = if pulse1 + pulse2 > 0.0 {
                95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0)
            } else {
                0.0
            };
            
            let tnd_out = if triangle + noise + dmc > 0.0 {
                159.79 / ((1.0 / ((triangle / 8227.0) + (noise / 12241.0) + (dmc / 22638.0))) + 100.0)
            } else {
                0.0
            };
            
            // Mix and normalize
            let mixed = pulse_out + tnd_out;
            
            // Apply filtering
            let filtered = self.filter.process(mixed);
            
            // Resample to output rate
            let mut resampled = Vec::new();
            self.resampler.process(filtered, &mut resampled);
            
            // Add to output buffer
            for sample in resampled {
                self.samples.push(sample);
                self.output_buffer.write(&[sample]);
            }
        }
    }
    
    fn get_pulse_output(&self, channel: &PulseChannel) -> f32 {
        if !channel.enabled || channel.length_counter == 0 {
            return 0.0;
        }
        
        if channel.timer_period < 8 || channel.timer_period > 0x7FF {
            return 0.0;
        }
        
        const DUTY_TABLE: [[u8; 8]; 4] = [
            [0, 1, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 0, 0, 0],
            [1, 0, 0, 1, 1, 1, 1, 1],
        ];
        
        let duty_value = DUTY_TABLE[channel.duty as usize][channel.duty_position as usize];
        if duty_value == 0 {
            return 0.0;
        }
        
        let volume = if channel.constant_volume {
            channel.volume
        } else {
            channel.envelope_value
        };
        
        volume as f32
    }
    
    fn get_triangle_output(&self) -> f32 {
        if !self.triangle.enabled || self.triangle.length_counter == 0 || self.triangle.linear_counter == 0 {
            return 0.0;
        }
        
        if self.triangle.timer_period < 2 {
            return 0.0;
        }
        
        const TRIANGLE_TABLE: [u8; 32] = [
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];
        
        TRIANGLE_TABLE[self.triangle.sequence_position as usize] as f32
    }
    
    fn get_noise_output(&self) -> f32 {
        if !self.noise.enabled || self.noise.length_counter == 0 {
            return 0.0;
        }
        
        if self.noise.shift_register & 1 != 0 {
            return 0.0;
        }
        
        let volume = if self.noise.constant_volume {
            self.noise.volume
        } else {
            self.noise.envelope_value
        };
        
        volume as f32
    }
    
    pub fn get_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }
    
    /// Get samples from the output buffer with proper timing
    pub fn read_samples(&mut self, output: &mut [f32]) -> usize {
        self.output_buffer.read(output)
    }
    
    /// Get current audio buffer statistics
    pub fn get_buffer_stats(&self) -> buffer::AdaptiveBufferStats {
        self.output_buffer.stats()
    }
    
    /// Set audio quality
    pub fn set_quality(&mut self, quality: ResamplerQuality) {
        let cpu_rate = 1789773.0;
        self.resampler = Resampler::new(quality, cpu_rate, self.sample_rate as f32);
    }
    
    /// Reset audio processing
    pub fn reset_audio(&mut self) {
        self.filter.reset();
        self.resampler.reset();
        self.samples.clear();
    }
}
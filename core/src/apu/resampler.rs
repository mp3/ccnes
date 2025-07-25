/// Audio resampler for NES APU
/// Converts from NES native sample rate to target output sample rate

use std::collections::VecDeque;

/// Linear interpolation resampler
/// Simple but effective for most use cases
#[derive(Debug, Clone)]
pub struct LinearResampler {
    source_rate: f32,
    target_rate: f32,
    phase: f32,
    prev_sample: f32,
}

impl LinearResampler {
    pub fn new(source_rate: f32, target_rate: f32) -> Self {
        Self {
            source_rate,
            target_rate,
            phase: 0.0,
            prev_sample: 0.0,
        }
    }
    
    pub fn process(&mut self, input: f32, output: &mut Vec<f32>) {
        let ratio = self.source_rate / self.target_rate;
        
        while self.phase < 1.0 {
            // Linear interpolation between previous and current sample
            let interpolated = self.prev_sample + (input - self.prev_sample) * self.phase;
            output.push(interpolated);
            self.phase += ratio;
        }
        
        self.phase -= 1.0;
        self.prev_sample = input;
    }
    
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.prev_sample = 0.0;
    }
}

/// Hermite interpolation resampler
/// Higher quality than linear, good balance of quality and performance
#[derive(Debug, Clone)]
pub struct HermiteResampler {
    source_rate: f32,
    target_rate: f32,
    phase: f32,
    history: [f32; 4],
}

impl HermiteResampler {
    pub fn new(source_rate: f32, target_rate: f32) -> Self {
        Self {
            source_rate,
            target_rate,
            phase: 0.0,
            history: [0.0; 4],
        }
    }
    
    pub fn process(&mut self, input: f32, output: &mut Vec<f32>) {
        // Shift history buffer
        self.history[0] = self.history[1];
        self.history[1] = self.history[2];
        self.history[2] = self.history[3];
        self.history[3] = input;
        
        let ratio = self.source_rate / self.target_rate;
        
        while self.phase < 1.0 {
            let interpolated = Self::hermite_interpolate(
                self.history[0],
                self.history[1],
                self.history[2],
                self.history[3],
                self.phase,
            );
            output.push(interpolated);
            self.phase += ratio;
        }
        
        self.phase -= 1.0;
    }
    
    fn hermite_interpolate(y0: f32, y1: f32, y2: f32, y3: f32, x: f32) -> f32 {
        // 4-point, 3rd-order Hermite interpolation
        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);
        
        ((c3 * x + c2) * x + c1) * x + c0
    }
    
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.history = [0.0; 4];
    }
}

/// Blep (Band-Limited Step) resampler
/// Highest quality, reduces aliasing artifacts
#[derive(Debug, Clone)]
pub struct BlepResampler {
    source_rate: f32,
    target_rate: f32,
    phase: f32,
    blep_buffer: VecDeque<f32>,
    prev_sample: f32,
}

impl BlepResampler {
    const BLEP_SIZE: usize = 16;
    const BLEP_SCALE: f32 = 0.9;
    
    pub fn new(source_rate: f32, target_rate: f32) -> Self {
        Self {
            source_rate,
            target_rate,
            phase: 0.0,
            blep_buffer: VecDeque::with_capacity(Self::BLEP_SIZE * 2),
            prev_sample: 0.0,
        }
    }
    
    pub fn process(&mut self, input: f32, output: &mut Vec<f32>) {
        let ratio = self.source_rate / self.target_rate;
        
        // Detect discontinuity
        let delta = input - self.prev_sample;
        if delta.abs() > 0.1 {
            // Add BLEP for discontinuity
            self.add_blep(self.phase, delta);
        }
        
        while self.phase < 1.0 {
            let mut sample = self.prev_sample + (input - self.prev_sample) * self.phase;
            
            // Apply BLEP correction
            if !self.blep_buffer.is_empty() {
                sample += self.blep_buffer.pop_front().unwrap_or(0.0);
            }
            
            output.push(sample);
            self.phase += ratio;
        }
        
        self.phase -= 1.0;
        self.prev_sample = input;
    }
    
    fn add_blep(&mut self, phase: f32, amplitude: f32) {
        // Simplified BLEP - in practice, would use a pre-computed table
        let blep_phase = phase * Self::BLEP_SIZE as f32;
        let start_idx = blep_phase as usize;
        
        for i in 0..Self::BLEP_SIZE {
            let t = (i as f32 - blep_phase + start_idx as f32) / Self::BLEP_SIZE as f32;
            if t >= 0.0 && t < 1.0 {
                let blep_value = Self::compute_blep(t) * amplitude * Self::BLEP_SCALE;
                
                let idx = i;
                if idx < self.blep_buffer.len() {
                    self.blep_buffer[idx] += blep_value;
                } else {
                    self.blep_buffer.push_back(blep_value);
                }
            }
        }
    }
    
    fn compute_blep(t: f32) -> f32 {
        // Polynomial approximation of band-limited step
        if t < 0.0 {
            0.0
        } else if t > 1.0 {
            1.0
        } else {
            let t2 = t * t;
            let t3 = t2 * t;
            let t4 = t2 * t2;
            let t5 = t3 * t2;
            
            0.5 * t5 - 2.5 * t4 + 5.0 * t3 - 5.0 * t2 + 2.5 * t
        }
    }
    
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.blep_buffer.clear();
        self.prev_sample = 0.0;
    }
}

/// Resampler quality settings
#[derive(Debug, Clone, Copy)]
pub enum ResamplerQuality {
    Low,      // Linear interpolation
    Medium,   // Hermite interpolation
    High,     // BLEP resampling
}

/// Unified resampler interface
#[derive(Debug, Clone)]
pub enum Resampler {
    Linear(LinearResampler),
    Hermite(HermiteResampler),
    Blep(BlepResampler),
}

impl Resampler {
    pub fn new(quality: ResamplerQuality, source_rate: f32, target_rate: f32) -> Self {
        match quality {
            ResamplerQuality::Low => Resampler::Linear(LinearResampler::new(source_rate, target_rate)),
            ResamplerQuality::Medium => Resampler::Hermite(HermiteResampler::new(source_rate, target_rate)),
            ResamplerQuality::High => Resampler::Blep(BlepResampler::new(source_rate, target_rate)),
        }
    }
    
    pub fn process(&mut self, input: f32, output: &mut Vec<f32>) {
        match self {
            Resampler::Linear(r) => r.process(input, output),
            Resampler::Hermite(r) => r.process(input, output),
            Resampler::Blep(r) => r.process(input, output),
        }
    }
    
    pub fn reset(&mut self) {
        match self {
            Resampler::Linear(r) => r.reset(),
            Resampler::Hermite(r) => r.reset(),
            Resampler::Blep(r) => r.reset(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_linear_resampler() {
        let mut resampler = LinearResampler::new(48000.0, 44100.0);
        let mut output = Vec::new();
        
        resampler.process(1.0, &mut output);
        assert!(!output.is_empty());
    }
    
    #[test]
    fn test_hermite_resampler() {
        let mut resampler = HermiteResampler::new(48000.0, 44100.0);
        let mut output = Vec::new();
        
        // Feed some samples to fill history
        for i in 0..10 {
            output.clear();
            resampler.process(i as f32 * 0.1, &mut output);
        }
        
        assert!(!output.is_empty());
    }
    
    #[test]
    fn test_resampler_quality_selection() {
        let _low = Resampler::new(ResamplerQuality::Low, 48000.0, 44100.0);
        let _med = Resampler::new(ResamplerQuality::Medium, 48000.0, 44100.0);
        let _high = Resampler::new(ResamplerQuality::High, 48000.0, 44100.0);
    }
}
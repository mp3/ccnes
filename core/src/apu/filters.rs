/// Audio filters for NES APU
/// Implements low-pass and high-pass filters for accurate NES audio reproduction

/// First-order low-pass filter
/// Used to simulate the analog characteristics of the NES audio output
#[derive(Debug, Clone)]
pub struct LowPassFilter {
    cutoff_freq: f32,
    sample_rate: f32,
    prev_output: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(cutoff_freq: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);
        
        Self {
            cutoff_freq,
            sample_rate,
            prev_output: 0.0,
            alpha,
        }
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        // y[n] = α * x[n] + (1 - α) * y[n-1]
        self.prev_output = self.alpha * input + (1.0 - self.alpha) * self.prev_output;
        self.prev_output
    }
    
    pub fn reset(&mut self) {
        self.prev_output = 0.0;
    }
    
    pub fn set_cutoff(&mut self, cutoff_freq: f32) {
        self.cutoff_freq = cutoff_freq;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / self.sample_rate;
        self.alpha = dt / (rc + dt);
    }
}

/// First-order high-pass filter
/// Used to remove DC offset from the audio signal
#[derive(Debug, Clone)]
pub struct HighPassFilter {
    cutoff_freq: f32,
    sample_rate: f32,
    prev_input: f32,
    prev_output: f32,
    alpha: f32,
}

impl HighPassFilter {
    pub fn new(cutoff_freq: f32, sample_rate: f32) -> Self {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);
        
        Self {
            cutoff_freq,
            sample_rate,
            prev_input: 0.0,
            prev_output: 0.0,
            alpha,
        }
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        // y[n] = α * (y[n-1] + x[n] - x[n-1])
        self.prev_output = self.alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output
    }
    
    pub fn reset(&mut self) {
        self.prev_input = 0.0;
        self.prev_output = 0.0;
    }
    
    pub fn set_cutoff(&mut self, cutoff_freq: f32) {
        self.cutoff_freq = cutoff_freq;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_freq);
        let dt = 1.0 / self.sample_rate;
        self.alpha = rc / (rc + dt);
    }
}

/// Combined filter chain for NES audio
/// Applies both high-pass and low-pass filtering in sequence
#[derive(Debug, Clone)]
pub struct NesAudioFilter {
    high_pass1: HighPassFilter,
    high_pass2: HighPassFilter,
    low_pass: LowPassFilter,
}

impl NesAudioFilter {
    pub fn new(sample_rate: f32) -> Self {
        // NES filter characteristics based on hardware analysis
        // First high-pass: 90 Hz (DC blocking)
        // Second high-pass: 440 Hz (additional filtering)
        // Low-pass: 14 kHz (anti-aliasing and analog simulation)
        Self {
            high_pass1: HighPassFilter::new(90.0, sample_rate),
            high_pass2: HighPassFilter::new(440.0, sample_rate),
            low_pass: LowPassFilter::new(14000.0, sample_rate),
        }
    }
    
    pub fn process(&mut self, input: f32) -> f32 {
        // Apply filters in sequence
        let hp1 = self.high_pass1.process(input);
        let hp2 = self.high_pass2.process(hp1);
        let output = self.low_pass.process(hp2);
        
        // Soft clipping to prevent harsh distortion
        if output > 1.0 {
            1.0 - (1.0 - output).abs().powf(0.7)
        } else if output < -1.0 {
            -1.0 + (1.0 + output).abs().powf(0.7)
        } else {
            output
        }
    }
    
    pub fn reset(&mut self) {
        self.high_pass1.reset();
        self.high_pass2.reset();
        self.low_pass.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_low_pass_filter() {
        let mut filter = LowPassFilter::new(1000.0, 44100.0);
        
        // Test impulse response
        let output1 = filter.process(1.0);
        assert!(output1 > 0.0 && output1 < 1.0);
        
        let output2 = filter.process(0.0);
        assert!(output2 > 0.0 && output2 < output1);
    }
    
    #[test]
    fn test_high_pass_filter() {
        let mut filter = HighPassFilter::new(100.0, 44100.0);
        
        // Test step response (DC should be blocked)
        for _ in 0..1000 {
            filter.process(1.0);
        }
        
        let steady_state = filter.process(1.0);
        assert!(steady_state.abs() < 0.1); // Should block DC
    }
    
    #[test]
    fn test_nes_audio_filter() {
        let mut filter = NesAudioFilter::new(44100.0);
        
        // Test clipping
        let clipped = filter.process(2.0);
        assert!(clipped <= 1.0);
        
        let clipped_neg = filter.process(-2.0);
        assert!(clipped_neg >= -1.0);
    }
}
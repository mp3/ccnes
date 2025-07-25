/// Audio buffer management for NES APU
/// Handles ring buffer, synchronization, and timing

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Ring buffer for audio samples
#[derive(Debug, Clone)]
pub struct AudioRingBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Write samples to the buffer
    pub fn write(&mut self, samples: &[f32]) -> usize {
        let available = self.capacity - self.buffer.len();
        let to_write = samples.len().min(available);
        
        self.buffer.extend(&samples[..to_write]);
        to_write
    }
    
    /// Read samples from the buffer
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let to_read = output.len().min(self.buffer.len());
        
        for i in 0..to_read {
            output[i] = self.buffer.pop_front().unwrap_or(0.0);
        }
        
        to_read
    }
    
    /// Get number of samples available to read
    pub fn available(&self) -> usize {
        self.buffer.len()
    }
    
    /// Get free space in buffer
    pub fn free_space(&self) -> usize {
        self.capacity - self.buffer.len()
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// Get the fill level as a percentage
    pub fn fill_level(&self) -> f32 {
        (self.buffer.len() as f32 / self.capacity as f32) * 100.0
    }
}

/// Thread-safe audio buffer for cross-thread communication
pub struct ThreadSafeAudioBuffer {
    buffer: Arc<Mutex<AudioRingBuffer>>,
}

impl ThreadSafeAudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(AudioRingBuffer::new(capacity))),
        }
    }
    
    /// Write samples (producer side)
    pub fn write(&self, samples: &[f32]) -> Result<usize, &'static str> {
        match self.buffer.lock() {
            Ok(mut buffer) => Ok(buffer.write(samples)),
            Err(_) => Err("Failed to lock buffer for writing"),
        }
    }
    
    /// Read samples (consumer side)
    pub fn read(&self, output: &mut [f32]) -> Result<usize, &'static str> {
        match self.buffer.lock() {
            Ok(mut buffer) => Ok(buffer.read(output)),
            Err(_) => Err("Failed to lock buffer for reading"),
        }
    }
    
    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        match self.buffer.lock() {
            Ok(buffer) => BufferStats {
                available: buffer.available(),
                free_space: buffer.free_space(),
                fill_level: buffer.fill_level(),
                is_full: buffer.is_full(),
                is_empty: buffer.is_empty(),
            },
            Err(_) => BufferStats::default(),
        }
    }
    
    /// Clone for sharing between threads
    pub fn clone(&self) -> Self {
        Self {
            buffer: Arc::clone(&self.buffer),
        }
    }
}

#[derive(Debug, Default)]
pub struct BufferStats {
    pub available: usize,
    pub free_space: usize,
    pub fill_level: f32,
    pub is_full: bool,
    pub is_empty: bool,
}

/// Dynamic buffer size adjustment based on timing
#[derive(Debug, Clone)]
pub struct AdaptiveBuffer {
    buffer: AudioRingBuffer,
    target_latency_ms: f32,
    sample_rate: f32,
    min_size: usize,
    max_size: usize,
    
    // Statistics for adaptation
    underrun_count: u32,
    overrun_count: u32,
    last_adjustment: std::time::Instant,
}

impl AdaptiveBuffer {
    pub fn new(sample_rate: f32, target_latency_ms: f32) -> Self {
        let target_size = ((sample_rate * target_latency_ms / 1000.0) as usize).next_power_of_two();
        let min_size = target_size / 2;
        let max_size = target_size * 4;
        
        Self {
            buffer: AudioRingBuffer::new(target_size),
            target_latency_ms,
            sample_rate,
            min_size,
            max_size,
            underrun_count: 0,
            overrun_count: 0,
            last_adjustment: std::time::Instant::now(),
        }
    }
    
    pub fn write(&mut self, samples: &[f32]) -> usize {
        if self.buffer.is_full() {
            self.overrun_count += 1;
            self.check_resize();
        }
        
        self.buffer.write(samples)
    }
    
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let read = self.buffer.read(output);
        
        if read < output.len() {
            // Underrun - fill rest with silence
            for i in read..output.len() {
                output[i] = 0.0;
            }
            self.underrun_count += 1;
            self.check_resize();
        }
        
        read
    }
    
    fn check_resize(&mut self) {
        // Only adjust every second to avoid thrashing
        if self.last_adjustment.elapsed().as_secs() < 1 {
            return;
        }
        
        let current_size = self.buffer.capacity;
        let mut new_size = current_size;
        
        // Adjust based on error rate
        if self.underrun_count > 5 {
            // Too many underruns - increase buffer
            new_size = (current_size * 3 / 2).min(self.max_size);
        } else if self.overrun_count > 5 && self.underrun_count == 0 {
            // Only overruns - decrease buffer for lower latency
            new_size = (current_size * 2 / 3).max(self.min_size);
        }
        
        if new_size != current_size {
            // Create new buffer with adjusted size
            let mut new_buffer = AudioRingBuffer::new(new_size);
            
            // Transfer existing samples
            let mut temp = vec![0.0; self.buffer.available()];
            self.buffer.read(&mut temp);
            new_buffer.write(&temp);
            
            self.buffer = new_buffer;
            self.underrun_count = 0;
            self.overrun_count = 0;
            self.last_adjustment = std::time::Instant::now();
        }
    }
    
    pub fn stats(&self) -> AdaptiveBufferStats {
        AdaptiveBufferStats {
            current_size: self.buffer.capacity,
            fill_level: self.buffer.fill_level(),
            underrun_count: self.underrun_count,
            overrun_count: self.overrun_count,
            target_latency_ms: self.target_latency_ms,
        }
    }
}

#[derive(Debug)]
pub struct AdaptiveBufferStats {
    pub current_size: usize,
    pub fill_level: f32,
    pub underrun_count: u32,
    pub overrun_count: u32,
    pub target_latency_ms: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ring_buffer() {
        let mut buffer = AudioRingBuffer::new(100);
        
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(buffer.write(&samples), 5);
        assert_eq!(buffer.available(), 5);
        
        let mut output = vec![0.0; 3];
        assert_eq!(buffer.read(&mut output), 3);
        assert_eq!(output, vec![1.0, 2.0, 3.0]);
        assert_eq!(buffer.available(), 2);
    }
    
    #[test]
    fn test_thread_safe_buffer() {
        let buffer = ThreadSafeAudioBuffer::new(1000);
        
        let samples = vec![1.0; 100];
        assert!(buffer.write(&samples).is_ok());
        
        let stats = buffer.stats();
        assert_eq!(stats.available, 100);
    }
    
    #[test]
    fn test_adaptive_buffer() {
        let mut buffer = AdaptiveBuffer::new(44100.0, 20.0);
        
        // Simulate normal operation
        let samples = vec![0.5; 100];
        buffer.write(&samples);
        
        let mut output = vec![0.0; 50];
        buffer.read(&mut output);
        
        let stats = buffer.stats();
        assert!(stats.underrun_count == 0);
    }
}
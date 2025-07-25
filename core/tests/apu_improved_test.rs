use ccnes_core::apu::{Apu, ResamplerQuality};

#[test]
fn test_improved_audio_generation() {
    let mut apu = Apu::with_sample_rate(44100);
    
    // Enable all channels
    apu.write_register(0x4015, 0x1F);
    
    // Configure pulse 1 - middle C (261.63 Hz)
    let period = (1789773.0 / (261.63 * 16.0)) as u16;
    apu.write_register(0x4000, 0x8F); // Duty 2, max volume
    apu.write_register(0x4002, (period & 0xFF) as u8);
    apu.write_register(0x4003, ((period >> 8) & 0x07) as u8);
    
    // Configure pulse 2 - E (329.63 Hz)
    let period2 = (1789773.0 / (329.63 * 16.0)) as u16;
    apu.write_register(0x4004, 0x8F); // Duty 2, max volume
    apu.write_register(0x4006, (period2 & 0xFF) as u8);
    apu.write_register(0x4007, ((period2 >> 8) & 0x07) as u8);
    
    // Generate samples for 100ms
    let samples_needed = 44100 / 10; // 100ms at 44.1kHz
    let mut total_samples = 0;
    
    while total_samples < samples_needed {
        for _ in 0..1000 {
            apu.step();
        }
        
        let samples = apu.get_samples();
        total_samples += samples.len();
    }
    
    assert!(total_samples > 0);
}

#[test]
fn test_audio_filtering() {
    let mut apu = Apu::with_sample_rate(48000);
    
    // Enable noise channel for testing filtering
    apu.write_register(0x4015, 0x08);
    apu.write_register(0x400C, 0x3F); // Max volume
    apu.write_register(0x400E, 0x03); // High frequency noise
    apu.write_register(0x400F, 0x00); // Length counter
    
    // Generate some samples
    for _ in 0..10000 {
        apu.step();
    }
    
    let samples = apu.get_samples();
    
    // Check that samples are within valid range
    for &sample in &samples {
        assert!(sample >= -1.0 && sample <= 1.0);
    }
}

#[test]
fn test_resampler_quality_settings() {
    let mut apu = Apu::new();
    
    // Test different quality settings
    apu.set_quality(ResamplerQuality::Low);
    apu.set_quality(ResamplerQuality::Medium);
    apu.set_quality(ResamplerQuality::High);
    
    // Generate a test tone
    apu.write_register(0x4015, 0x01);
    apu.write_register(0x4000, 0x8F);
    apu.write_register(0x4002, 0x00);
    apu.write_register(0x4003, 0x01);
    
    for _ in 0..1000 {
        apu.step();
    }
    
    assert!(!apu.get_samples().is_empty());
}

#[test]
fn test_buffer_management() {
    let mut apu = Apu::with_sample_rate(44100);
    
    // Enable triangle for smooth waveform
    apu.write_register(0x4015, 0x04);
    apu.write_register(0x4008, 0x81); // Linear counter
    apu.write_register(0x400A, 0x20); // Low frequency
    apu.write_register(0x400B, 0x00);
    
    // Test reading samples from buffer
    let mut output = vec![0.0; 1024];
    
    // Generate samples
    for _ in 0..50000 {
        apu.step();
    }
    
    let read = apu.read_samples(&mut output);
    assert!(read > 0);
    
    // Check buffer statistics
    let stats = apu.get_buffer_stats();
    assert!(stats.current_size > 0);
}

#[test]
fn test_audio_reset() {
    let mut apu = Apu::new();
    
    // Generate some audio
    apu.write_register(0x4015, 0x01);
    apu.write_register(0x4000, 0x8F);
    apu.write_register(0x4002, 0x10);
    apu.write_register(0x4003, 0x00);
    
    for _ in 0..1000 {
        apu.step();
    }
    
    // Reset audio processing
    apu.reset_audio();
    
    // Samples should be cleared
    let samples = apu.get_samples();
    assert!(samples.is_empty());
}

#[test]
fn test_mixed_channel_output() {
    let mut apu = Apu::with_sample_rate(44100);
    
    // Enable all channels
    apu.write_register(0x4015, 0x1F);
    
    // Configure each channel with different frequencies
    // Pulse 1: 440 Hz (A4)
    let p1_period = (1789773.0 / (440.0 * 16.0)) as u16;
    apu.write_register(0x4000, 0x88); // 50% duty, volume 8
    apu.write_register(0x4002, (p1_period & 0xFF) as u8);
    apu.write_register(0x4003, ((p1_period >> 8) & 0x07) as u8);
    
    // Triangle: 220 Hz (A3)
    let tri_period = (1789773.0 / (220.0 * 32.0)) as u16;
    apu.write_register(0x4008, 0x81);
    apu.write_register(0x400A, (tri_period & 0xFF) as u8);
    apu.write_register(0x400B, ((tri_period >> 8) & 0x07) as u8);
    
    // Generate mixed output
    for _ in 0..10000 {
        apu.step();
    }
    
    let samples = apu.get_samples();
    
    // Verify we got samples and they're properly mixed
    assert!(!samples.is_empty());
    
    // Calculate RMS to ensure signal has energy
    let rms: f32 = samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32;
    let rms = rms.sqrt();
    
    assert!(rms > 0.01); // Should have some signal energy
    assert!(rms < 1.0);  // Should not be clipping
}
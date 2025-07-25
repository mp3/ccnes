use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ccnes_core::{Nes, cartridge::Cartridge, cpu::CpuBus};

fn create_test_rom() -> Vec<u8> {
    let mut rom_data = vec![0; 16 + 0x8000]; // Header + 32KB PRG ROM
    
    // iNES header
    rom_data[0..4].copy_from_slice(b"NES\x1A");
    rom_data[4] = 2; // 2 PRG ROM banks
    rom_data[5] = 0; // No CHR ROM
    rom_data[6] = 0x00; // Mapper 0
    rom_data[7] = 0x00;
    
    // Test program - busy loop with various instructions
    let mut offset = 16;
    
    // Initialize
    rom_data[offset] = 0xA9; offset += 1; // LDA #$00
    rom_data[offset] = 0x00; offset += 1;
    rom_data[offset] = 0x85; offset += 1; // STA $00
    rom_data[offset] = 0x00; offset += 1;
    
    // Main loop
    let loop_start = offset;
    rom_data[offset] = 0xE6; offset += 1; // INC $00
    rom_data[offset] = 0x00; offset += 1;
    rom_data[offset] = 0xA5; offset += 1; // LDA $00
    rom_data[offset] = 0x00; offset += 1;
    rom_data[offset] = 0xC9; offset += 1; // CMP #$FF
    rom_data[offset] = 0xFF; offset += 1;
    rom_data[offset] = 0xD0; offset += 1; // BNE loop
    rom_data[offset] = (loop_start as i32 - offset as i32 - 1) as u8; offset += 1;
    
    // Reset after loop
    rom_data[offset] = 0xA9; offset += 1; // LDA #$00
    rom_data[offset] = 0x00; offset += 1;
    rom_data[offset] = 0x85; offset += 1; // STA $00
    rom_data[offset] = 0x00; offset += 1;
    rom_data[offset] = 0x4C; offset += 1; // JMP loop_start
    rom_data[offset] = (loop_start & 0xFF) as u8; offset += 1;
    rom_data[offset] = (0x80 | (loop_start >> 8)) as u8; offset += 1;
    
    // Reset vector
    rom_data[16 + 0x7FFC] = 0x00;
    rom_data[16 + 0x7FFD] = 0x80;
    
    rom_data
}

fn benchmark_frame_execution(c: &mut Criterion) {
    c.bench_function("frame_execution", |b| {
        b.iter_batched(
            || {
                let rom_data = create_test_rom();
                let cartridge = Cartridge::from_ines(&rom_data[..]).unwrap();
                let mut nes = Nes::new();
                nes.load_cartridge(cartridge);
                nes
            },
            |mut nes| {
                nes.run_frame();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn benchmark_cpu_step(c: &mut Criterion) {
    c.bench_function("cpu_step_1000", |b| {
        b.iter_batched(
            || {
                let rom_data = create_test_rom();
                let cartridge = Cartridge::from_ines(&rom_data[..]).unwrap();
                let mut nes = Nes::new();
                nes.load_cartridge(cartridge);
                nes
            },
            |mut nes| {
                for _ in 0..1000 {
                    nes.step();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn benchmark_ppu_rendering(c: &mut Criterion) {
    c.bench_function("ppu_scanline", |b| {
        b.iter_batched(
            || {
                let rom_data = create_test_rom();
                let cartridge = Cartridge::from_ines(&rom_data[..]).unwrap();
                let mut nes = Nes::new();
                nes.load_cartridge(cartridge);
                // Enable rendering
                nes.bus.write(0x2001, 0x18); // Show background and sprites
                nes
            },
            |mut nes| {
                // Run for approximately one scanline worth of cycles
                for _ in 0..114 {
                    nes.step();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn benchmark_memory_access(c: &mut Criterion) {
    c.bench_function("memory_read_write", |b| {
        b.iter_batched(
            || {
                let rom_data = create_test_rom();
                let cartridge = Cartridge::from_ines(&rom_data[..]).unwrap();
                let mut nes = Nes::new();
                nes.load_cartridge(cartridge);
                nes
            },
            |mut nes| {
                // Benchmark various memory accesses
                for addr in (0..256).step_by(16) {
                    black_box(nes.bus.read(addr));
                    nes.bus.write(addr, addr as u8);
                }
                
                // ROM reads
                for addr in (0x8000..0x8100).step_by(16) {
                    black_box(nes.bus.read(addr));
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    benchmark_frame_execution,
    benchmark_cpu_step,
    benchmark_ppu_rendering,
    benchmark_memory_access
);
criterion_main!(benches);
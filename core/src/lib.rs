pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod cartridge;
pub mod controller;
pub mod bus;
pub mod nes;

pub mod test_rom;

pub use cpu::Cpu;
pub use ppu::Ppu;
pub use apu::Apu;
pub use cartridge::Cartridge;
pub use controller::{Controller, ControllerButton};
pub use bus::Bus;
pub use nes::Nes;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub cpu_cycles: u64,
    pub ppu_cycles: u64,
    pub apu_cycles: u64,
}
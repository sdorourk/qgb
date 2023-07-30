mod bits;
pub mod cartridge;
mod components;
pub mod cpu;

use std::fmt::Debug;

use thiserror::Error;

use components::mmu;

pub type TCycles = i64;

pub struct GameBoy {
    pub(crate) cpu: cpu::Cpu<mmu::Mmu>,
}

#[derive(Debug, Error)]
pub enum BootError {
    #[error("{0}")]
    RomError(#[from] RomError),
    #[error("{0}")]
    BootRomError(#[from] BootRomError),
}

#[derive(Debug, Error)]
pub enum RomError {
    #[error("unexpected ROM file size (expected at least {expected} bytes, found {found} bytes")]
    Undersized { expected: usize, found: usize },
    #[error("cartridge header: unrecognized cartridge type ({0:02X})")]
    UnrecognizedCartridgeType(u8),
    #[error("cartridge header: unrecognized ROM size ({0:02X})")]
    UnrecognizedRomSize(u8),
    #[error("cartridge header: unrecognized RAM size ({0:02X})")]
    UnrecognizedRamSize(u8),
    #[error("unsupported cartridge type (found '{0:?}' cartridge type)")]
    UnsupportedCartridgeType(cartridge::CartridgeType),
    #[error(
        "unexpected ROM file size for cartridge type `{cartridge_type:?}' (found {found} bytes) "
    )]
    Oversized {
        cartridge_type: cartridge::CartridgeType,
        found: usize,
    },
    #[error("unexpected ROM file size (expected {expected} bytes, found {found} bytes")]
    Size { expected: usize, found: usize },
}

#[derive(Debug, Error)]
pub enum BootRomError {
    #[error("unexpected boot ROM size (expected {expected} bytes, found {found} bytes")]
    Size { expected: usize, found: usize },
}

impl GameBoy {
    pub fn new(rom: &[u8], boot_rom: &[u8]) -> Result<Self, BootError> {
        let mmu = mmu::Mmu::new(rom, boot_rom)?;

        Ok(Self {
            cpu: cpu::Cpu::new(mmu),
        })
    }

    pub fn cpu(&self) -> &cpu::Cpu<mmu::Mmu> {
        &self.cpu
    }

    pub fn fetch(&self) -> Result<cpu::instruction::Instruction, u8> {
        self.cpu.fetch()
    }

    pub fn step(&mut self) -> TCycles {
        self.cpu.step()
    }
}

mod bits;
pub mod cartridge;
pub mod cpu;
pub mod mmu;

use thiserror::Error;

pub type TCycles = i64;

pub struct GameBoy {
    pub(crate) cpu: cpu::Cpu,
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
}

#[derive(Debug, Error)]
pub enum BootRomError {
    #[error("unexpected boot ROM size (expected {expected} bytes, found {found} bytes")]
    Size { expected: usize, found: usize },
}

impl GameBoy {
    pub fn new(rom: &[u8], boot_rom: &[u8]) -> Result<Self, BootError> {
        let header = cartridge::Header::try_from(rom)?;
        tracing::debug!(target: "boot", cartridge_header = ?header);

        let mmu = mmu::Mmu::new(boot_rom)?;

        Ok(Self {
            cpu: cpu::Cpu::new(mmu),
        })
    }

    pub fn cpu(&self) -> &cpu::Cpu {
        &self.cpu
    }
}

use thiserror::Error;

use std::fmt::Debug;

use crate::cartridge;

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

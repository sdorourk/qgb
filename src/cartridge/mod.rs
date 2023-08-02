mod cartridge_base;
mod header;
mod mbc1;
mod rom_only;

use std::fmt::Debug;

pub use header::*;

use crate::RomError;

const ROM_BANK_SIZE: usize = 16 * 1024;
const RAM_BANK_SIZE: usize = 8 * 1024;

/// Game cartridge
pub type Cartridge = Box<dyn CartridgeInterface + Send + Sync>;

/// The `CartridgeInterface` trait needs to be implemented for each supported
/// cartridge type.
pub trait CartridgeInterface: Debug {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, value: u8);
    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, value: u8);
    fn header(&self) -> &Header;
}

pub fn new_cartridge(rom: &[u8]) -> Result<Cartridge, RomError> {
    let header = Header::parse(rom)?;
    tracing::debug!(target: "boot", cartridge_header = ?header);

    if header.rom_banks * ROM_BANK_SIZE != rom.len() {
        return Err(RomError::Size {
            expected: header.rom_banks * ROM_BANK_SIZE,
            found: rom.len(),
        });
    }

    match header.cartridge_type {
        CartridgeType::RomOnly => Ok(Box::new(rom_only::RomOnly::new(rom, header)?)),
        CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => {
            // TODO: detect MBC1M multi-cart
            Ok(Box::new(mbc1::Mbc1::new(rom, header)?))
        }
        _ => Err(RomError::UnsupportedCartridgeType(header.cartridge_type)),
    }
}

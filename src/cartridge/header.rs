use std::fmt::Debug;
use std::num::Wrapping;

use crate::RomError;

const MIN_CARTRIDGE_SIZE: usize = 32 * 1024;
const CARTRIDGE_TITLE_START: usize = 0x0134;
const CARTRIDGE_TITLE_END: usize = 0x0143;
const CARTRIDGE_TYPE: usize = 0x0147;
const ROM_SIZE: usize = 0x0148;
const RAM_SIZE: usize = 0x0149;
const HEADER_CHECKSUM: usize = 0x014D;

/// Cartridge header
#[derive(Debug)]
pub struct Header {
    /// Cartridge title
    pub title: String,
    /// Cartridge type
    pub cartridge_type: CartridgeType,
    /// Number of ROM banks
    pub rom_banks: usize,
    /// Number of RAM banks
    pub ram_banks: usize,
    /// Header checksum
    pub checksum: u8,
    /// Header checksum matches computed value
    pub checksum_passed: bool,
}

impl Header {
    pub fn parse(rom: &[u8]) -> Result<Self, RomError> {
        if rom.len() < MIN_CARTRIDGE_SIZE {
            return Err(RomError::Undersized {
                expected: MIN_CARTRIDGE_SIZE,
                found: rom.len(),
            });
        }

        let mut title = String::new();
        for byte in &rom[CARTRIDGE_TITLE_START..=CARTRIDGE_TITLE_END] {
            let ch = *byte as char;
            if *byte == 0 || !ch.is_ascii() {
                break;
            }
            title.push(ch);
        }

        let cartridge_type = CartridgeType::try_from(rom[CARTRIDGE_TYPE])?;

        let rom_banks: usize = {
            if rom[ROM_SIZE] <= 0x08 {
                2 * (1 << rom[ROM_SIZE])
            } else {
                return Err(RomError::UnrecognizedRomSize(rom[ROM_SIZE]));
            }
        };

        let ram_banks: usize = {
            match rom[RAM_SIZE] {
                0x00 => 0,
                0x02 => 1,
                0x03 => 4,
                0x04 => 16,
                0x05 => 8,
                _ => return Err(RomError::UnrecognizedRamSize(rom[RAM_SIZE])),
            }
        };

        let checksum = rom[HEADER_CHECKSUM];
        let mut computed_checksum: Wrapping<u8> = Wrapping(0);
        for byte in &rom[CARTRIDGE_TITLE_START..HEADER_CHECKSUM] {
            computed_checksum = computed_checksum - Wrapping(*byte) - Wrapping(1);
        }

        Ok(Self {
            title,
            cartridge_type,
            rom_banks,
            ram_banks,
            checksum,
            checksum_passed: checksum == computed_checksum.0,
        })
    }
}

#[derive(Debug)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    RomRam,
    RomRamBattery,
    Mmm01,
    Mmm01Ram,
    Mmm01RamBattery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    Mbc6,
    Mbc7SensorRumbleRamBattery,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1RamBattery,
}

impl TryFrom<u8> for CartridgeType {
    type Error = RomError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::RomOnly),
            0x01 => Ok(Self::Mbc1),
            0x02 => Ok(Self::Mbc1Ram),
            0x03 => Ok(Self::Mbc1RamBattery),
            0x05 => Ok(Self::Mbc2),
            0x06 => Ok(Self::Mbc2Battery),
            0x08 => Ok(Self::RomRam),
            0x09 => Ok(Self::RomRamBattery),
            0x0B => Ok(Self::Mmm01),
            0x0C => Ok(Self::Mmm01Ram),
            0x0D => Ok(Self::Mmm01RamBattery),
            0x0F => Ok(Self::Mbc3TimerBattery),
            0x10 => Ok(Self::Mbc3TimerRamBattery),
            0x11 => Ok(Self::Mbc3),
            0x12 => Ok(Self::Mbc3Ram),
            0x13 => Ok(Self::Mbc3RamBattery),
            0x19 => Ok(Self::Mbc5),
            0x1A => Ok(Self::Mbc5Ram),
            0x1B => Ok(Self::Mbc5RamBattery),
            0x1C => Ok(Self::Mbc5Rumble),
            0x1D => Ok(Self::Mbc5RumbleRam),
            0x1E => Ok(Self::Mbc5RumbleRamBattery),
            0x20 => Ok(Self::Mbc6),
            0x22 => Ok(Self::Mbc7SensorRumbleRamBattery),
            0xFC => Ok(Self::PocketCamera),
            0xFD => Ok(Self::BandaiTama5),
            0xFE => Ok(Self::HuC3),
            0xFF => Ok(Self::HuC1RamBattery),
            _ => Err(RomError::UnrecognizedCartridgeType(value)),
        }
    }
}

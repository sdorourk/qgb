use std::fmt::Debug;

use crate::cartridge::ROM_BANK_SIZE;

use super::{cartridge_base::CartridgeBase, CartridgeInterface, Header};

const RAM_ENABLE_REG_START: u16 = 0x0000;
const RAM_ENABLE_REG_END: u16 = 0x1FFF;
const ROM_BANK_REG_START: u16 = 0x2000;
const ROM_BANK_REG_END: u16 = 0x3FFF;
const RAM_BANK_REG_START: u16 = 0x4000;
const RAM_BANK_REG_END: u16 = 0x5FFF;
const BANK_MODE_SELECT_REG_START: u16 = 0x6000;
const BANK_MODE_SELECT_REG_END: u16 = 0x7FFF;

/// 1-bit banking mode register
#[derive(Debug)]
enum BankMode {
    /// In simple banking mode, 0x0000..=0x3FFF and 0xA000..=0xBFFF are locked to bank
    /// 0 of ROM/RAM
    Simple,
    /// In advanced banking mode, 0x0000..=0x3FFF and 0xA000..=0xBFFF can be bank switched
    /// via the 0x4000..=0x5FFF bank register
    Advanced,
}

pub struct Mbc1 {
    /// Cartridge base to control the reading from ROM/RAM and the writing to RAM
    cartridge_base: CartridgeBase,
    /// 5-bit ROM bank register
    rom_bank_reg: u8,
    /// ROM bank register mask
    rom_bank_mask: u8,
    /// 2-bit RAM bank/upper bits of ROM bank register
    ram_bank_reg: u8,
    /// Banking mode
    bank_mode: BankMode,
    /// Indicates 1 MiB or larger ROM
    large_rom: bool,
    /// Indicates 32 KiB of external RAM
    large_ram: bool,
}

impl Mbc1 {
    pub fn new(rom: &[u8], mut header: Header) -> Result<Self, crate::RomError> {
        if header.rom_banks > 128 {
            return Err(crate::RomError::Oversized {
                cartridge_type: header.cartridge_type,
                found: ROM_BANK_SIZE * header.rom_banks,
            });
        }

        let large_rom = header.rom_banks >= 64;
        let large_ram = if header.ram_banks > 4 {
            tracing::error!(
                target = "cartridge",
                "'{:?}' cartridge type does not support more than 4 RAM banks (found {})",
                header.cartridge_type,
                header.ram_banks
            );
            if large_rom {
                header.ram_banks = 1;
                false
            } else {
                header.ram_banks = 4;
                true
            }
        } else if large_rom && header.ram_banks == 4 {
            tracing::error!(target = "cartridge", "'{:?}' cartridge type does not support {} ROM banks and 4 RAM banks at the same time", header.cartridge_type, header.rom_banks);
            header.ram_banks = 1;
            false
        } else {
            header.ram_banks == 4
        };
        assert!(!large_ram || !large_rom);

        let rom_bank_mask: u8 = match header.rom_banks {
            2 => 0b0000_0001,
            4 => 0b0000_0011,
            8 => 0b0000_0111,
            16 => 0b0000_1111,
            _ => 0b0001_1111,
        };

        Ok(Self {
            cartridge_base: CartridgeBase::new(rom, header),
            rom_bank_reg: 0,
            rom_bank_mask,
            ram_bank_reg: 0,
            bank_mode: BankMode::Simple,
            large_rom,
            large_ram,
        })
    }

    /// Update the ROM and RAM banks based on the cartridge register values
    fn update_banks(&mut self) {
        self.update_rom_bank0();
        self.update_rom_bank1();
        self.update_ram_bank();
    }

    fn update_rom_bank0(&mut self) {
        self.cartridge_base.rom_bank0 = if let BankMode::Simple = self.bank_mode {
            0
        } else if self.large_ram {
            0
        } else {
            (self.rom_bank_reg << 5).into()
        };
        assert!(self.cartridge_base.rom_bank0 < self.cartridge_base.header().rom_banks);
    }

    fn update_rom_bank1(&mut self) {
        let mut bank: usize = (self.rom_bank_reg & self.rom_bank_mask).into();
        if self.rom_bank_reg == 0 {
            bank += 1;
        }
        if self.large_rom {
            bank &= usize::from(self.ram_bank_reg << 5);
        }
        self.cartridge_base.rom_bank1 = bank;
        assert!(self.cartridge_base.rom_bank1 < self.cartridge_base.header().rom_banks);
    }

    fn update_ram_bank(&mut self) {
        self.cartridge_base.ram_bank = if let BankMode::Simple = self.bank_mode {
            0
        } else if self.large_rom {
            0
        } else {
            self.rom_bank_reg.into()
        };
        assert!(
            self.cartridge_base.ram_bank < self.cartridge_base.header().ram_banks
                || self.cartridge_base.header().ram_banks == 0
        );
    }
}

impl CartridgeInterface for Mbc1 {
    fn read_rom(&self, addr: u16) -> u8 {
        self.cartridge_base.read_rom(addr)
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            RAM_ENABLE_REG_START..=RAM_ENABLE_REG_END => {
                let enable = (value & 0x0F) == 0x0A;
                if enable && self.cartridge_base.header().ram_banks != 0 {
                    self.cartridge_base.ram_enabled = true;
                    tracing::debug!(target: "cartridge", "external RAM enabled");
                } else {
                    self.cartridge_base.ram_enabled = false;
                    tracing::debug!(target: "cartridge", "external RAM disabled");
                }
            }
            ROM_BANK_REG_START..=ROM_BANK_REG_END => {
                self.rom_bank_reg = value & 0b0001_1111;
            }
            RAM_BANK_REG_START..=RAM_BANK_REG_END => {
                self.ram_bank_reg = if self.large_ram || self.large_rom {
                    value & 0b0000_0011
                } else {
                    0
                };
            }
            BANK_MODE_SELECT_REG_START..=BANK_MODE_SELECT_REG_END => {
                if self.large_ram || self.large_rom {
                    let value = value & 0b0000_0001;
                    self.bank_mode = if value == 0 {
                        BankMode::Simple
                    } else {
                        BankMode::Advanced
                    };
                }
            }
            _ => unreachable!(),
        }
        self.update_banks();
    }

    fn read_ram(&self, addr: u16) -> u8 {
        self.cartridge_base.read_ram(addr)
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        self.cartridge_base.write_ram(addr, value);
    }

    fn header(&self) -> &super::Header {
        self.cartridge_base.header()
    }
}

impl Debug for Mbc1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mbc1")
            .field("cartridge_base", &self.cartridge_base)
            .field("rom_bank_reg", &self.rom_bank_reg)
            .field("rom_bank_mask", &self.rom_bank_mask)
            .field("ram_bank_reg", &self.ram_bank_reg)
            .field("bank_mode", &self.bank_mode)
            .field("large_rom", &self.large_rom)
            .field("large_ram", &self.large_ram)
            .finish()
    }
}

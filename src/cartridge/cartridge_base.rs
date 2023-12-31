use crate::{
    components::mmu::{ROM_BANK0_END, ROM_BANK0_START, ROM_BANK1_END, ROM_BANK1_START},
    state::{CartridgeState, PollState},
};

use super::{RAM_BANK_SIZE, ROM_BANK_SIZE};

use std::fmt::Debug;

const DEFAULT_READ_VALUE: u8 = 0;

/// Base implementation for a game cartridge
pub struct CartridgeBase {
    /// Cartridge ROM
    pub rom: Box<[u8]>,
    /// Index for the 16 KiB ROM bank mapped to memory 0x0000..=0x3FFF
    pub rom_bank0: usize,
    /// Index for the 16 KiB ROM bank mapped to memory 0x4000..=0x7FFF
    pub rom_bank1: usize,
    /// External (cartridge) RAM
    pub ram: Option<Box<[u8]>>,
    /// Index for the 8 KiB external RAM mapped to memory 0xA000..=0xBFFF
    pub ram_bank: usize,
    /// External RAM can typically be enabled/disabled
    pub ram_enabled: bool,
    header: super::Header,
}

impl CartridgeBase {
    pub fn new(rom: &[u8], header: super::Header) -> Self {
        let ram = match header.ram_banks {
            0 => None,
            ram_banks => Some(vec![0; RAM_BANK_SIZE * ram_banks].into_boxed_slice()),
        };
        Self {
            rom: rom.into(),
            rom_bank0: 0,
            rom_bank1: 1,
            ram,
            ram_bank: 0,
            ram_enabled: false,
            header,
        }
    }
}

impl Debug for CartridgeBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CartridgeBase")
            .field("rom_bank0", &self.rom_bank0)
            .field("rom_bank1", &self.rom_bank1)
            .field("ram_bank", &self.ram_bank)
            .field("ram_enabled", &self.ram_enabled)
            .field("header", &self.header)
            .finish_non_exhaustive()
    }
}

impl super::CartridgeInterface for CartridgeBase {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            ROM_BANK0_START..=ROM_BANK0_END => {
                assert!(self.rom_bank0 < self.header.rom_banks);
                let addr: usize = (addr - ROM_BANK0_START).into();
                self.rom[ROM_BANK_SIZE * self.rom_bank0 + addr]
            }
            ROM_BANK1_START..=ROM_BANK1_END => {
                assert!(self.rom_bank1 < self.header.rom_banks);
                let addr: usize = (addr - ROM_BANK1_START).into();
                self.rom[ROM_BANK_SIZE * self.rom_bank1 + addr]
            }
            _ => unreachable!(),
        }
    }

    fn write_rom(&mut self, _addr: u16, _value: u8) {
        tracing::warn!(target: "cartridge", "attempted to write to cartridge ROM: cartridge does not support writing to ROM");
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if let Some(ram) = &self.ram {
            if self.ram_enabled {
                assert!(self.ram_bank < self.header.ram_banks);
                let addr: usize = addr.into();
                ram[RAM_BANK_SIZE * self.ram_bank + addr]
            } else {
                tracing::error!(target: "cartridge", "attempted to read from external RAM, but RAM is not enabled");
                DEFAULT_READ_VALUE
            }
        } else {
            tracing::error!(target: "cartridge", "attempted to read from non-existent external RAM");
            DEFAULT_READ_VALUE
        }
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if let Some(ram) = &mut self.ram {
            if self.ram_enabled {
                assert!(self.ram_bank < self.header.ram_banks);
                let addr: usize = addr.into();
                ram[RAM_BANK_SIZE * self.ram_bank + addr] = value;
            } else {
                tracing::error!(target: "cartridge", "attempted to write to external RAM, but RAM is not enabled");
            }
        } else {
            tracing::error!(target: "cartridge", "attempted to write to non-existent external RAM");
        }
    }

    fn header(&self) -> &super::Header {
        &self.header
    }
}

impl PollState for CartridgeBase {
    fn poll_state(&self, state: &mut crate::State) {
        if state.cartridge.is_none() {
            state.cartridge = Some(CartridgeState {
                header: self.header.clone(),
                rom: self.rom.clone(),
                ram: self.ram.clone(),
                ..Default::default()
            });
        }
        if let Some(cart_state) = &mut state.cartridge {
            cart_state.rom_bank0 = self.rom_bank0;
            cart_state.rom_bank0_range =
                (self.rom_bank0 * ROM_BANK_SIZE)..((self.rom_bank0 + 1) * ROM_BANK_SIZE);
            cart_state.rom_bank1 = self.rom_bank1;
            cart_state.rom_bank1_range =
                (self.rom_bank1 * ROM_BANK_SIZE)..((self.rom_bank1 + 1) * ROM_BANK_SIZE);
            cart_state.ram_enabled = self.ram_enabled;
            if self.ram.is_some() {
                cart_state.ram_bank_range =
                    Some((self.ram_bank * RAM_BANK_SIZE)..((self.ram_bank + 1) * RAM_BANK_SIZE));
            }
        }
    }
}

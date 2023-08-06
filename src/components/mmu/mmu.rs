use std::fmt::Debug;

use super::*;

use crate::{cartridge, components::io::IoHandler, state::PollState, TCycles};

const BOOT_ROM_SIZE: usize = 0x0100;

const DEFAULT_READ_VALUE: u8 = 0xFF;

pub const ROM_BANK0_START: u16 = 0x0000;
pub const ROM_BANK0_END: u16 = 0x3FFF;
pub const ROM_BANK1_START: u16 = 0x4000;
pub const ROM_BANK1_END: u16 = 0x7FFF;
const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;
pub const EXTERNAL_RAM_START: u16 = 0xA000;
const EXTERNAL_RAM_END: u16 = 0xBFFF;
pub const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;
const WRAM_SIZE: usize = (WRAM_END - WRAM_START + 1) as usize;
const MIRROR_WRAM_START: u16 = 0xE000;
const MIRROR_WRAM_END: u16 = 0xFDFF;
const OAM_START: u16 = 0xFE00;
const OAM_END: u16 = 0xFE9F;
const IO_REG_START: u16 = 0xFF00;
const IO_REG_END: u16 = 0xFF02;
const TIMER_REG_START: u16 = 0xFF04;
const TIMER_REG_END: u16 = 0xFF07;
const INTERRUPT_FLAG: u16 = 0xFF0F;
pub const APU_CHANNEL1_SWEEP: u16 = 0xFF10;
pub const APU_CHANNEL1_PERIOD_HIGH: u16 = 0xFF14;
pub const APU_CHANNEL2_LENGTH: u16 = 0xFF16;
pub const APU_CHANNEL3_PERIOD_HIGH: u16 = 0xFF1E;
pub const APU_CHANNEL4_LENGTH: u16 = 0xFF20;
pub const APU_SOUND_ON_OFF: u16 = 0xFF26;
pub const APU_STORAGE_START: u16 = 0xFF30;
pub const APU_STORAGE_END: u16 = 0xFF3F;
pub const PPU_REG_START: u16 = 0xFF40;
pub const PPU_REG_END: u16 = 0xFF4B;
const BANK_REG: u16 = 0xFF50;
pub const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFE;
const HRAM_SIZE: usize = (HRAM_END - HRAM_START + 1) as usize;
pub const INTERRUPT_ENABLE_REG: u16 = 0xFFFF;

/// Memory management unit
pub struct Mmu {
    /// Boot ROM
    boot_rom: Box<[u8]>,
    /// In boot mode, the boot ROM is mapped over the cartridge ROM
    boot_mode: bool,
    /// Game cartridge
    cartridge: cartridge::Cartridge,
    /// High RAM
    hram: [u8; HRAM_SIZE],
    /// Work RAM
    wram: [u8; WRAM_SIZE],
    /// Joypad and serial transfer input/output
    io: IoHandler,
}

impl Mmu {
    pub fn new(rom: &[u8], boot_rom: &[u8]) -> Result<Self, crate::BootError> {
        if boot_rom.len() != BOOT_ROM_SIZE {
            return Err(crate::BootRomError::Size {
                expected: BOOT_ROM_SIZE,
                found: boot_rom.len(),
            }
            .into());
        }
        Ok(Self {
            boot_rom: boot_rom.into(),
            boot_mode: true,
            cartridge: cartridge::new_cartridge(rom)?,
            hram: [0; HRAM_SIZE],
            wram: [0; WRAM_SIZE],
            io: IoHandler::new(),
        })
    }
}

impl Debug for Mmu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mmu")
            .field("boot_mode", &self.boot_mode)
            .field("cartridge", &self.cartridge)
            .finish_non_exhaustive()
    }
}

/// Memory mapped devices/locations
#[derive(Debug)]
enum MappedAddress {
    /// Cartridge ROM
    CartridgeRom,
    /// Video RAM
    VRam(u16),
    /// External (cartridge) RAM
    ExternalRam(u16),
    /// Work RAM
    WRam(u16),
    /// Mirrored work RAM
    MirrorRam(u16),
    /// Object attribute memory
    Oam(u16),
    /// I/O registers
    IoReg,
    /// Timer registers
    TimerReg,
    /// Audio processing unit registers
    ApuReg,
    /// Pixel processing unit registers
    PpuReg,
    /// Bank register, used to indicate when the boot ROM should be unmapped
    BankReg,
    /// High RAM
    HRam(u16),
    /// Interrupt flag and register
    Interrupt,
}

impl TryFrom<u16> for MappedAddress {
    type Error = ();

    fn try_from(addr: u16) -> Result<Self, Self::Error> {
        match addr {
            ROM_BANK0_START..=ROM_BANK1_END => Ok(Self::CartridgeRom),
            VRAM_START..=VRAM_END => Ok(Self::VRam(addr - VRAM_START)),
            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => {
                Ok(Self::ExternalRam(addr - EXTERNAL_RAM_START))
            }
            WRAM_START..=WRAM_END => Ok(Self::WRam(addr - WRAM_START)),
            MIRROR_WRAM_START..=MIRROR_WRAM_END => Ok(Self::MirrorRam(addr - MIRROR_WRAM_START)),
            OAM_START..=OAM_END => Ok(Self::Oam(addr - OAM_START)),
            IO_REG_START..=IO_REG_END => Ok(Self::IoReg),
            TIMER_REG_START..=TIMER_REG_END => Ok(Self::TimerReg),
            INTERRUPT_FLAG => Ok(Self::Interrupt),
            APU_CHANNEL1_SWEEP..=APU_CHANNEL1_PERIOD_HIGH
            | APU_CHANNEL2_LENGTH..=APU_SOUND_ON_OFF
            | APU_STORAGE_START..=APU_STORAGE_END => Ok(Self::ApuReg),
            PPU_REG_START..=PPU_REG_END => Ok(Self::PpuReg),
            BANK_REG => Ok(Self::BankReg),
            HRAM_START..=HRAM_END => Ok(Self::HRam(addr - HRAM_START)),
            INTERRUPT_ENABLE_REG => Ok(Self::Interrupt),
            _ => Err(()),
        }
    }
}

/// Information returned after a byte has been read from memory
#[derive(Debug)]
struct ReadInfo {
    /// The value read from memory
    value: u8,
    /// The mapped address
    mapped_addr: MappedAddress,
    /// Byte read from the boot ROM
    boot_rom_read: bool,
}

/// Information returned after a byte has been written to memory
#[derive(Debug)]
struct WriteInfo {
    /// The mapped address
    mapped_addr: MappedAddress,
    /// Write disabled the boot ROM
    boot_rom_disabled: bool,
}

impl Mmu {
    /// Attempt to read from the given address.
    ///
    /// Return `None` if the address could not be read from (e.g., if the address
    /// corresponds to a register for the CGB).
    fn raw_read(&self, addr: u16) -> Option<ReadInfo> {
        let mapped_addr = match MappedAddress::try_from(addr) {
            Err(_) | Ok(MappedAddress::BankReg) => {
                return None;
            }
            Ok(mapped_addr) => mapped_addr,
        };

        // Determine if we are reading from the mapped over boot ROM
        let boot_rom_read = self.boot_mode && usize::from(addr) < BOOT_ROM_SIZE;

        let value = match mapped_addr {
            MappedAddress::CartridgeRom => {
                if boot_rom_read {
                    self.boot_rom[usize::from(addr)]
                } else {
                    self.cartridge.read_rom(addr)
                }
            }
            // MappedAddress::VRam(_) => todo!(),
            MappedAddress::ExternalRam(addr) => self.cartridge.read_ram(addr),
            MappedAddress::WRam(addr) => self.wram[usize::from(addr)],
            // MappedAddress::MirrorRam(_) => todo!(),
            // MappedAddress::Oam(_) => todo!(),
            MappedAddress::IoReg => self.io.read(addr),
            // MappedAddress::TimerReg => todo!(),
            // MappedAddress::ApuReg => todo!(),
            // MappedAddress::PpuReg => todo!(),
            MappedAddress::BankReg => unreachable!(),
            MappedAddress::HRam(addr) => self.hram[usize::from(addr)],
            // MappedAddress::Interrupt => todo!(),
            _ => return None,
        };

        Some(ReadInfo {
            value,
            mapped_addr,
            boot_rom_read,
        })
    }

    /// Attempt to write to the given address.
    ///
    /// Return `Err` if the address can not be written to (e.g., if the address corresponds
    /// to a register for the CGB).
    fn raw_write(&mut self, addr: u16, value: u8) -> Result<WriteInfo, ()> {
        let mapped_addr = match MappedAddress::try_from(addr) {
            Ok(mapped_addr) => mapped_addr,
            Err(_) => {
                return Err(());
            }
        };

        let mut boot_rom_disabled = false;

        match mapped_addr {
            MappedAddress::CartridgeRom => self.cartridge.write_rom(addr, value),
            // MappedAddress::VRam(_) => todo!(),
            MappedAddress::ExternalRam(addr) => self.cartridge.write_ram(addr, value),
            MappedAddress::WRam(addr) => self.wram[usize::from(addr)] = value,
            // MappedAddress::MirrorRam(_) => todo!(),
            // MappedAddress::Oam(_) => todo!(),
            MappedAddress::IoReg => self.io.write(addr, value),
            // MappedAddress::TimerReg => todo!(),
            // MappedAddress::ApuReg => todo!(),
            // MappedAddress::PpuReg => todo!(),
            MappedAddress::BankReg => {
                if value != 0 {
                    self.boot_mode = false;
                    boot_rom_disabled = true;
                }
            }
            MappedAddress::HRam(addr) => self.hram[usize::from(addr)] = value,
            // MappedAddress::Interrupt => todo!(),
            _ => return Err(()),
        };
        Ok(WriteInfo {
            mapped_addr,
            boot_rom_disabled,
        })
    }
}

impl ReadWriteMemory for Mmu {
    fn read(&self, addr: u16) -> u8 {
        match self.raw_read(addr) {
            Some(info) => {
                if tracing::enabled!(target: "mmu", tracing::Level::TRACE) {
                    let mapped_to = if info.boot_rom_read {
                        "BootRom".to_string()
                    } else {
                        format!("{:?}", info.mapped_addr)
                    };
                    tracing::trace!(target: "mmu", "read ${:02X} from memory address ${addr:04X} (mapped to {})", info.value, mapped_to);
                }
                info.value
            }
            None => {
                tracing::error!(target: "mmu", "attempted to read from unmapped memory address ${:04X}", addr);
                DEFAULT_READ_VALUE
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match self.raw_write(addr, value) {
            Ok(info) => {
                tracing::trace!(target: "mmu", "wrote ${value:02X} to memory address ${addr:04X} (mapped to {:?})", info.mapped_addr);
                if info.boot_rom_disabled {
                    tracing::trace!(target: "mmu", "boot mode disabled")
                }
            }
            Err(()) => {
                tracing::error!(target: "mmu", "attempted to write to unmapped memory address ${:04X}", addr);
            }
        }
    }
}

impl Tick for Mmu {
    fn tick(&mut self, cycles: TCycles) {
        self.io.tick(cycles);
    }
}

impl PollState for Mmu {
    fn poll_state(&self, state: &mut crate::State) {
        if state.mmu.is_none() {
            state.mmu = Some(Default::default());
        }
        if let Some(mmu_state) = &mut state.mmu {
            mmu_state.boot_mode = self.boot_mode;
            mmu_state.boot_rom = self.boot_rom.clone();
            mmu_state.wram = self.wram.into();
            mmu_state.hram = self.hram.into();
        }
        self.cartridge.poll_state(state);
        self.io.poll_state(state);
    }
}

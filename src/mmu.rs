const BOOT_ROM_SIZE: usize = 0x0100;

/// Memory management unit
#[derive(Debug)]
pub struct Mmu {
    /// Boot ROM
    boot_rom: Box<[u8]>,
    /// In boot mode, the boot ROM is mapped over the cartridge ROM
    boot_mode: bool,
}

impl Mmu {
    pub fn new(boot_rom: &[u8]) -> Result<Self, crate::BootRomError> {
        if boot_rom.len() != BOOT_ROM_SIZE {
            return Err(crate::BootRomError::Size {
                expected: BOOT_ROM_SIZE,
                found: boot_rom.len(),
            });
        }
        Ok(Self {
            boot_rom: boot_rom.into(),
            boot_mode: true,
        })
    }
}

/// The `ReadWriteMemory` trait allows for reading and writing bytes to memory.
pub trait ReadWriteMemory {
    /// Read a byte from the given address
    fn read(&self, addr: u16) -> u8;

    /// Write `value` to the given memory address
    fn write(&mut self, addr: u16, value: u8);

    /// Read two bytes from the given address
    fn read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr.wrapping_add(1))])
    }

    /// Write `value` to the given memory address
    fn write_u16(&mut self, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.write(addr, bytes[0]);
        self.write(addr.wrapping_add(1), bytes[1]);
    }
}

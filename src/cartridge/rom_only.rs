use super::{cartridge_base::CartridgeBase, Header, ROM_BANK_SIZE};

/// RomOnly game cartridge (sometimes referred to as MBC0)
#[derive(Debug)]
pub struct RomOnly {
    cartridge_base: CartridgeBase,
}

impl RomOnly {
    pub fn new(rom: &[u8], header: Header) -> Result<Self, crate::RomError> {
        if header.rom_banks != 2 {
            return Err(crate::RomError::Oversized {
                cartridge_type: header.cartridge_type,
                found: ROM_BANK_SIZE * header.rom_banks,
            });
        }

        let ram_banks = header.ram_banks;
        if ram_banks > 1 {
            tracing::error!(target: "cartridge", "'{:?}' cartridge type does not support more than one RAM bank (found {} banks)", header.cartridge_type, header.ram_banks);
        }
        let mut cartridge_base = CartridgeBase::new(rom, header);
        if ram_banks != 0 {
            cartridge_base.ram_enabled = true;
        }
        Ok(Self { cartridge_base })
    }
}

impl super::CartridgeInterface for RomOnly {
    fn read_rom(&self, addr: u16) -> u8 {
        self.cartridge_base.read_rom(addr)
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        self.cartridge_base.write_rom(addr, value)
    }

    fn read_ram(&self, addr: u16) -> u8 {
        self.cartridge_base.read_ram(addr)
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        self.cartridge_base.write_ram(addr, value)
    }

    fn header(&self) -> &Header {
        self.cartridge_base.header()
    }
}

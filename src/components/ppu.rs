use super::mmu::{OAM_SIZE, PPU_REG_END, PPU_REG_START, VRAM_SIZE};

const NUMBER_OF_PPU_REGISTERS: usize = (PPU_REG_END - PPU_REG_START + 1) as usize;

/// Todo: Add a real PPU implementation
#[derive(Debug)]
pub struct Ppu {
    reg: [u8; NUMBER_OF_PPU_REGISTERS],
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
}

impl Ppu {
    pub fn new() -> Self {
        let mut ppu = Self {
            reg: [0; NUMBER_OF_PPU_REGISTERS],
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
        };
        // TODO: Remove this line (it is used to indicate the VBlank period and prevents
        // the boot ROM from entering an infinite loop)
        ppu.reg[4] = 0x90;
        ppu
    }

    pub fn vram_read(&self, addr: u16) -> u8 {
        self.vram[usize::from(addr)]
    }

    pub fn vram_write(&mut self, addr: u16, value: u8) {
        self.vram[usize::from(addr)] = value;
    }

    pub fn oam_read(&self, addr: u16) -> u8 {
        self.oam[usize::from(addr)]
    }

    pub fn oam_write(&mut self, addr: u16, value: u8) {
        self.oam[usize::from(addr)] = value;
    }

    pub fn reg_read(&self, addr: u16) -> u8 {
        assert!(addr >= PPU_REG_START);
        assert!(addr <= PPU_REG_END);
        self.reg[usize::from(addr - PPU_REG_START)]
    }

    pub fn reg_write(&mut self, addr: u16, value: u8) {
        assert!(addr >= PPU_REG_START);
        assert!(addr <= PPU_REG_END);
        self.reg[usize::from(addr - PPU_REG_START)] = value;
    }
}

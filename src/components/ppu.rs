use crate::{
    bits::Bits,
    components::mmu::{
        PPU_BGP, PPU_LCDC, PPU_LY, PPU_LYC, PPU_OBP0, PPU_OBP1, PPU_SCX, PPU_SCY, PPU_STAT, PPU_WX,
        PPU_WY,
    },
    TCycles,
};

use super::mmu::{InterruptManager, OAM_SIZE, PPU_DMA, VRAM_SIZE};

pub const DISPLAY_WIDTH: usize = 160;
pub const DISPLAY_HEIGHT: usize = 144;
/// Tile size (in bytes)
const TILE_SIZE: usize = 16;
const TILE_MAP_WIDTH: usize = 32;
const TILE_MAP_HEIGHT: usize = 32;

/// Todo: Add a real PPU implementation
#[derive(Debug)]
pub struct Ppu {
    lcdc: Lcdc,
    stat: Stat,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    dma: u8,
    bgp: ColorPalette,
    obp0: ColorPalette,
    obp1: ColorPalette,
    wy: u8,
    wx: u8,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
}

impl Ppu {
    pub fn new() -> Self {
        let mut ppu = Self {
            lcdc: Default::default(),
            stat: Default::default(),
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: Default::default(),
            obp0: Default::default(),
            obp1: Default::default(),
            wy: 0,
            wx: 0,
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
        };
        // TODO: Remove this line (it is used to indicate the VBlank period and prevents
        // the boot ROM from entering an infinite loop)
        ppu.ly = 0x90;
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
        match addr {
            PPU_LCDC => self.lcdc.into(),
            PPU_STAT => self.stat.into(),
            PPU_SCY => self.scy,
            PPU_SCX => self.scx,
            PPU_LY => self.ly,
            PPU_LYC => self.lyc,
            PPU_DMA => self.dma,
            PPU_BGP => self.bgp.into(),
            PPU_OBP0 => self.obp0.into(),
            PPU_OBP1 => self.obp1.into(),
            PPU_WY => self.wy,
            PPU_WX => self.wx,
            _ => unreachable!(),
        }
    }

    pub fn reg_write(&mut self, addr: u16, value: u8) {
        match addr {
            PPU_LCDC => self.lcdc = value.into(),
            PPU_STAT => self.stat = value.into(),
            PPU_SCY => self.scy = value,
            PPU_SCX => self.scx = value,
            PPU_LY => {
                tracing::error!(target: "ppu", "cannot write to LY register");
            }
            PPU_LYC => self.lyc = value,
            PPU_DMA => self.dma = value,
            PPU_BGP => self.bgp = value.into(),
            PPU_OBP0 => self.obp0 = value.into(),
            PPU_OBP1 => self.obp1 = value.into(),
            PPU_WY => self.wy = value,
            PPU_WX => self.wx = value,
            _ => unreachable!(),
        }
    }

    pub fn tick<T: InterruptManager>(&mut self, mut cycles: TCycles, _interrupt_manager: &mut T) {
        while cycles > 0 {
            cycles -= 1;
        }
    }

    pub fn screen(&self) -> Vec<Color> {
        let mut screen = Vec::with_capacity(DISPLAY_HEIGHT * DISPLAY_WIDTH);

        for y in 0..DISPLAY_HEIGHT as u8 {
            for x in 0..DISPLAY_WIDTH as u8 {
                screen
                    .push(self.bg_pixel_color(self.scx.wrapping_add(x), self.scy.wrapping_add(y)));
            }
        }

        screen
    }

    fn bg_pixel_color(&self, x: u8, y: u8) -> Color {
        if self.lcdc.lcd_enable {
            let x_offset = x % 8;
            let y_offset = y % 8;
            let x = usize::from(x / 8);
            let y = usize::from(y / 8);
            let map_addr = if self.lcdc.bg_tile_map_area {
                0x1C00 + (y * TILE_MAP_WIDTH) + x
            } else {
                0x1800 + (y * TILE_MAP_WIDTH) + x
            };

            let tile_index = self.vram[map_addr];
            self.bg_tile_pixel_color(tile_index, x_offset, y_offset)
        } else {
            Color::Black
        }
    }

    fn bg_tile_pixel_color(&self, tile_index: u8, x_offset: u8, y_offset: u8) -> Color {
        assert!(x_offset < 8);
        assert!(y_offset < 8);

        let x_offset = usize::from(x_offset);
        let y_offset = usize::from(y_offset);

        let tile_addr = if self.lcdc.bg_window_tile_data_area {
            usize::from(tile_index) * TILE_SIZE
        } else {
            0x0800 + usize::from(tile_index) * TILE_SIZE
        };

        let tile = &self.vram[tile_addr..tile_addr + TILE_SIZE];
        let lsb = tile[y_offset * 2].bit(7 - x_offset);
        let msb = tile[y_offset * 2 + 1].bit(7 - x_offset);
        let mut val = 0;
        if msb {
            val.set_bit(1);
        }
        if lsb {
            val.set_bit(0);
        }

        self.bgp.colors[usize::from(val)]
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Lcdc {
    /// LCD and PPU enable
    lcd_enable: bool,
    /// Window tile map area (false = 0x9800..=0x9BFF, true = 0x9C00..=0x9FFF)
    window_tile_map_area: bool,
    /// Window enable
    window_enable: bool,
    /// BG and window tile data areas (false = 0x8800..=0x97FF, true = 0x8000..=0x8FFF)
    bg_window_tile_data_area: bool,
    /// BG tile map area (false = 0x9800..=0x9BFF, true = 0x9C00..=0x9FFF)
    bg_tile_map_area: bool,
    /// OBJ size (false = 8x8, true = 8x16)
    obj_size: bool,
    /// OBJ enable
    obj_enable: bool,
    /// BG and window enable/priority
    bg_window_enable: bool,
}

impl From<u8> for Lcdc {
    fn from(value: u8) -> Self {
        Self {
            lcd_enable: value.bit(7),
            window_tile_map_area: value.bit(6),
            window_enable: value.bit(5),
            bg_window_tile_data_area: value.bit(4),
            bg_tile_map_area: value.bit(3),
            obj_size: value.bit(2),
            obj_enable: value.bit(1),
            bg_window_enable: value.bit(0),
        }
    }
}

impl From<Lcdc> for u8 {
    fn from(lcdc: Lcdc) -> Self {
        let mut value = 0;
        if lcdc.lcd_enable {
            value.set_bit(7);
        }
        if lcdc.window_tile_map_area {
            value.set_bit(6);
        }
        if lcdc.window_enable {
            value.set_bit(5);
        }
        if lcdc.bg_window_tile_data_area {
            value.set_bit(4);
        }
        if lcdc.bg_tile_map_area {
            value.set_bit(3);
        }
        if lcdc.obj_size {
            value.set_bit(2);
        }
        if lcdc.obj_enable {
            value.set_bit(1);
        }
        if lcdc.bg_window_enable {
            value.set_bit(0);
        }
        value
    }
}

#[derive(Debug, Clone, Copy)]
enum ModeFlag {
    HBlank,
    VBlank,
    SearchingOam,
    TransferringData,
}

impl Default for ModeFlag {
    fn default() -> Self {
        Self::HBlank
    }
}

impl From<u8> for ModeFlag {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::HBlank,
            1 => Self::VBlank,
            2 => Self::SearchingOam,
            3 => Self::TransferringData,
            _ => unreachable!(),
        }
    }
}

impl From<ModeFlag> for u8 {
    fn from(value: ModeFlag) -> Self {
        match value {
            ModeFlag::HBlank => 0,
            ModeFlag::VBlank => 1,
            ModeFlag::SearchingOam => 2,
            ModeFlag::TransferringData => 3,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Stat {
    /// LYC = LY STAT interrupt source
    lyc_interrupt_source: bool,
    /// Mode 2 OAM STAT interrupt source
    oam_interrupt_source: bool,
    /// Mode 1 VBlank STAT interrupt source
    vblank_interrupt_source: bool,
    /// Mode 0 HBlank STAT interrupt source
    hblank_interrupt_source: bool,
    /// LYC = LY flag (0 = different, 1 = equal) (Read only)
    lyc_flag: bool,
    /// Mode flag (0 = HBlank, 1 = VBlank, 2 = Searching OAM,
    /// 3 = Transferring data to LCD controller) (Read only)
    mode_flag: ModeFlag,
}

impl From<u8> for Stat {
    fn from(value: u8) -> Self {
        Self {
            lyc_interrupt_source: value.bit(6),
            oam_interrupt_source: value.bit(5),
            vblank_interrupt_source: value.bit(4),
            hblank_interrupt_source: value.bit(3),
            lyc_flag: value.bit(2),
            mode_flag: value.bits(0..=1).into(),
        }
    }
}

impl From<Stat> for u8 {
    fn from(stat: Stat) -> Self {
        let mut value: u8 = stat.mode_flag.into();
        if stat.lyc_flag {
            value.set_bit(2);
        }
        if stat.hblank_interrupt_source {
            value.set_bit(3);
        }
        if stat.vblank_interrupt_source {
            value.set_bit(4);
        }
        if stat.oam_interrupt_source {
            value.set_bit(5);
        }
        if stat.lyc_interrupt_source {
            value.set_bit(6);
        }
        value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Default for Color {
    fn default() -> Self {
        Self::White
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::White,
            1 => Self::LightGray,
            2 => Self::DarkGray,
            3 => Self::Black,
            _ => unreachable!(),
        }
    }
}

impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        match value {
            Color::White => 0,
            Color::LightGray => 1,
            Color::DarkGray => 2,
            Color::Black => 3,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct ColorPalette {
    colors: [Color; 4],
}

impl From<u8> for ColorPalette {
    fn from(value: u8) -> Self {
        let mut palette = Self::default();
        palette.colors[0] = value.bits(0..=1).into();
        palette.colors[1] = value.bits(2..=3).into();
        palette.colors[2] = value.bits(4..=5).into();
        palette.colors[3] = value.bits(6..=7).into();
        palette
    }
}

impl From<ColorPalette> for u8 {
    fn from(palette: ColorPalette) -> Self {
        u8::from(palette.colors[0])
            + (u8::from(palette.colors[1]) << 2)
            + (u8::from(palette.colors[2]) << 4)
            + (u8::from(palette.colors[3]) << 6)
    }
}

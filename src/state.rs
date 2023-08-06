//! Module for polling and recording the emulator's state

use std::{collections::HashMap, fmt::Debug, ops::RangeInclusive};

/// Default number of instructions to fetch each time the state of the emulator is polled
const DEFAULT_INSTRUCTION_BUFFER_SIZE: usize = 50;

/// The `PollState` trait allows for polling the emulator's state from its various
/// components.
pub trait PollState {
    /// Request `state` be updated with the latest state of the emulator
    fn poll_state(&self, state: &mut State);
}

/// Emulation state provided to the debugger
#[derive(Debug)]
pub struct State {
    /// Cartridge state
    pub cartridge: Option<CartridgeState>,
    /// CPU state
    pub cpu: Option<CpuState>,
    /// MMU state
    pub mmu: Option<MmuState>,
    /// Number of instructions to fetch each time the debugger polls the state of the
    /// emulator
    pub(crate) instruction_buffer_size: usize,
    /// State of the joypad and serial transfer registers
    pub io: Option<IoState>,
}

/// Cartridge state
pub struct CartridgeState {
    /// Cartridge title specified in the header
    pub title: String,
    /// Cartridge type
    pub cartridge_type: String,
    /// Number of ROM banks
    pub rom_banks: usize,
    /// Number of external RAM banks
    pub ram_banks: usize,
    /// Cartridge header checksum
    pub checksum: u8,
    /// Header checksum matches computed value
    pub checksum_passed: bool,
    /// Cartridge ROM
    pub rom: Box<[u8]>,
    /// Index of the first ROM bank
    pub rom_bank0: usize,
    /// Range of values from `rom` stored in the first ROM bank
    pub rom_bank0_range: RangeInclusive<usize>,
    /// Index of the second ROM bank
    pub rom_bank1: usize,
    /// Range of values from `rom` store in the second ROM bank
    pub rom_bank1_range: RangeInclusive<usize>,
    /// External (cartridge) RAM (if any)
    pub ram: Option<Box<u8>>,
    /// Index of the RAM bank
    pub ram_bank: usize,
    /// Range of value from `ram` accessible to the CPU
    pub ram_bank_range: Option<RangeInclusive<usize>>,
    /// External RAM enabled
    pub ram_enabled: bool,
    /// MBC-dependent state details.
    pub mbc_state: Option<HashMap<String, String>>,
}

/// MMU state
pub struct MmuState {
    pub boot_mode: bool,
    pub boot_rom: Box<u8>,
    pub wram: Box<u8>,
    pub hram: Box<u8>,
}

/// CPU state
#[derive(Default)]
pub struct CpuState {
    /// 8-bit register A
    pub a: u8,
    /// 8-bit register B
    pub b: u8,
    /// 8-bit register C
    pub c: u8,
    /// 8-bit register D
    pub d: u8,
    /// 8-bit register E
    pub e: u8,
    /// 8-bit flags register
    pub f: u8,
    /// 8-bit register H
    pub h: u8,
    /// 8-bit register L
    pub l: u8,
    /// 16-bit register AF
    pub af: u16,
    /// 16-bit register BC
    pub bc: u16,
    /// 16-bit register DE
    pub de: u16,
    /// 16-bit register HL
    pub hl: u16,
    /// 8-bit value accessed from the address specified in the `HL` register
    pub deref_hl: u8,
    /// Zero flag
    pub z_flag: bool,
    /// Subtraction flag
    pub n_flag: bool,
    /// Half carry flag
    pub h_flag: bool,
    /// Carry flag
    pub c_flag: bool,
    /// Program counter
    pub pc: u16,
    /// Stack pointer
    pub sp: u16,
    /// Instructions fetched from memory starting at the program counter
    pub instructions: Vec<InstructionInfo>,
}

/// Information concerning a fetched instruction
#[derive(Debug, Default)]
pub struct InstructionInfo {
    /// Instruction mnemonic
    pub display: String,
    /// Address in memory where the instruction opcode begins
    pub address: u16,
    /// Bytes from memory representing the instruction
    pub bytes: Vec<u8>,
}

/// State of the joypad and serial transfer registers
#[derive(Debug, Default)]
pub struct IoState {
    /// 8-bit registers
    pub registers: HashMap<u16, u8>,
    /// All bytes that have been transferred through the serial port
    pub transmitted_bytes: Vec<u8>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            cartridge: Default::default(),
            cpu: Default::default(),
            mmu: Default::default(),
            instruction_buffer_size: DEFAULT_INSTRUCTION_BUFFER_SIZE,
            io: Default::default(),
        }
    }
}

impl Debug for CartridgeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CartridgeState")
            .field("title", &self.title)
            .field("cartridge_type", &self.cartridge_type)
            .field("rom_banks", &self.rom_banks)
            .field("ram_banks", &self.ram_banks)
            .field("checksum", &self.checksum)
            .field("checksum_passed", &self.checksum_passed)
            .field("rom_bank0", &self.rom_bank0)
            .field("rom_bank0_range", &self.rom_bank0_range)
            .field("rom_bank1", &self.rom_bank1)
            .field("rom_bank1_range", &self.rom_bank1_range)
            .field("ram_bank", &self.ram_bank)
            .field("ram_bank_range", &self.ram_bank_range)
            .field("ram_enabled", &self.ram_enabled)
            .field("mbc_state", &self.mbc_state)
            .finish_non_exhaustive()
    }
}

impl Default for CartridgeState {
    fn default() -> Self {
        Self {
            title: Default::default(),
            cartridge_type: Default::default(),
            rom_banks: Default::default(),
            ram_banks: Default::default(),
            checksum: Default::default(),
            checksum_passed: Default::default(),
            rom: Default::default(),
            rom_bank0: Default::default(),
            rom_bank0_range: 0..=0,
            rom_bank1: Default::default(),
            rom_bank1_range: 0..=0,
            ram: Default::default(),
            ram_bank: Default::default(),
            ram_bank_range: Default::default(),
            ram_enabled: Default::default(),
            mbc_state: Default::default(),
        }
    }
}

impl Debug for MmuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MmuState")
            .field("boot_mode", &self.boot_mode)
            .finish_non_exhaustive()
    }
}

impl Debug for CpuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpuState")
            .field("a", &self.a)
            .field("b", &self.b)
            .field("c", &self.c)
            .field("d", &self.d)
            .field("e", &self.e)
            .field("f", &self.f)
            .field("h", &self.h)
            .field("l", &self.l)
            .field("af", &self.af)
            .field("bc", &self.bc)
            .field("de", &self.de)
            .field("hl", &self.hl)
            .field("z_flag", &self.z_flag)
            .field("n_flag", &self.n_flag)
            .field("h_flag", &self.h_flag)
            .field("c_flag", &self.c_flag)
            .field("pc", &self.pc)
            .field("sp", &self.sp)
            .finish_non_exhaustive()
    }
}

impl IoState {
    /// Interpret the transmitted bytes as ASCII characters
    pub fn transmitted_bytes_ascii(&self) -> String {
        let mut msg = String::new();
        for byte in &self.transmitted_bytes {
            let ch = *byte as char;
            if ch.is_ascii() {
                msg.push(ch);
            }
        }
        msg
    }
}

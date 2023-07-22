pub mod instruction;
pub mod opcode;

use bitflags::bitflags;

pub type TCycles = i64;

#[derive(Debug)]
pub struct Cpu {
    /// Register A
    pub a: u8,
    /// Register B
    pub b: u8,
    /// Register C
    pub c: u8,
    /// Register D
    pub d: u8,
    /// Register E
    pub e: u8,
    /// Flags register
    f: FlagsRegister,
    /// Register H
    pub h: u8,
    /// Register L
    pub l: u8,
    /// Stack pointer
    pub sp: u16,
    /// Program counter
    pub pc: u16,
}

bitflags! {
    /// Flags register (lower 8-bits of the AF register)
    #[derive(Debug, Default, Clone, Copy)]
    pub struct FlagsRegister: u8 {
        /// Zero flag
        const Z = 0b1000_0000;
        /// Subtraction flag
        const N = 0b0100_0000;
        /// Half carry flag
        const H = 0b0010_0000;
        /// Carry flag
        const C = 0b0001_0000;
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister::default(),
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
}

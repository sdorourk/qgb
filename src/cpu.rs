pub mod instruction;
pub mod opcode;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct FlagsRegister: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

pub type TCycles = i32;

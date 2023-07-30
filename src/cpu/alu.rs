use crate::components::mmu::{ReadWriteMemory, Tick};

use super::{
    opcode::{Register, WideRegister},
    Cpu, FlagsRegister,
};

use std::fmt::Debug;

impl<T> Cpu<T>
where
    T: Debug + ReadWriteMemory + Tick,
{
    /// Add two 8-bit registers together (re)setting the carry and half-carry flags
    /// as required.
    ///
    /// Returns the (wrapped) sum of the two registers.
    pub(super) fn add(&mut self, first: Register, second: Register) -> u8 {
        let x = self.reg(first);
        let y = self.reg(second);
        let hc = half_carry_add(x, y);
        let (value, carry) = x.overflowing_add(y);

        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);

        value
    }

    /// Add two 16-bit registers together (re)setting the carry and half-carry flags
    /// as required.
    ///
    /// Returns the (wrapped) sum of the two registers.
    pub(super) fn add_wide(&mut self, first: WideRegister, second: WideRegister) -> u16 {
        let x = self.wide_reg(first);
        let y = self.wide_reg(second);
        let hc = half_carry_add_wide(x, y);
        let (value, carry) = x.overflowing_add(y);

        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);

        value
    }

    /// Increment an 8-bit register (re)setting the zero and half-carry flags as
    /// required.
    pub(super) fn inc(&mut self, reg: Register) {
        let mut value = self.reg(reg);
        let hc = half_carry_add(value, 1);
        value = value.wrapping_add(1);
        self.set_reg(reg, value);

        self.f.set(FlagsRegister::Z, value == 0);
        self.f.set(FlagsRegister::H, hc);
    }

    /// Decrement an 8-bit register (re)setting the zero and half-carry flags as
    /// required.
    pub(super) fn dec(&mut self, reg: Register) {
        let mut value = self.reg(reg);
        let hc = half_carry_sub(value, 1);
        value = value.wrapping_sub(1);
        self.set_reg(reg, value);

        self.f.set(FlagsRegister::Z, value == 0);
        self.f.set(FlagsRegister::H, hc);
    }

    /// Increment a 16-bit register.
    ///
    /// No flags are (re)set when performing this operation.
    pub(super) fn inc_wide(&mut self, reg: WideRegister) {
        let mut value = self.wide_reg(reg);
        value = value.wrapping_add(1);
        self.set_wide_reg(reg, value);
    }

    /// Decrement a 16-bit register.
    ///
    /// No flags are (re)set when performing this operation.
    pub(super) fn dec_wide(&mut self, reg: WideRegister) {
        let mut value = self.wide_reg(reg);
        value = value.wrapping_sub(1);
        self.set_wide_reg(reg, value);
    }
}

/// Check if a half carry will occur when adding `x` and `y`.
fn half_carry_add(x: u8, y: u8) -> bool {
    (((x & 0x0F) + (y & 0x0F)) & 0x10) == 0x10
}

/// Check if a half carry will occur when subtracting `x` and `y`.
fn half_carry_sub(x: u8, y: u8) -> bool {
    ((x & 0x0F).wrapping_sub(y & 0x0F)) & 0x10 == 0x10
}

/// Check if a half carry will occur when adding `x` and `y`.
fn half_carry_add_wide(x: u16, y: u16) -> bool {
    (((x & 0x0FFF) + (y & 0x0FFF)) & 0x1000) == 0x1000
}

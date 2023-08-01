use crate::{
    bits::Bits,
    components::mmu::{ReadWriteMemory, Tick},
};

use super::{
    opcode::{Register, WideRegister},
    Cpu, FlagsRegister,
};

use std::fmt::Debug;

impl<T> Cpu<T>
where
    T: Debug + ReadWriteMemory + Tick,
{
    /// Add an 8-bit value to the accumulator register, (re)setting the carry,  
    /// half-carry, and zero flags as required.
    pub(super) fn add(&mut self, n: u8) {
        let value = self.reg(Register::A);
        let hc = half_carry_add(value, n);
        let (value, carry) = value.overflowing_add(n);

        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Add an 8-bit value to the accumulator register plus the carry flag, (re)setting
    /// the carry,  half-carry, and zero flags as required.
    pub(super) fn adc(&mut self, n: u8) {
        let value = self.reg(Register::A);
        let old_carry = self.f.intersects(FlagsRegister::C);

        let hc = half_carry_adc(value, n, old_carry);
        let (value, carry1) = value.overflowing_add(n);
        let (value, carry2) = value.overflowing_add(if old_carry { 1 } else { 0 });

        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry1 || carry2);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Subtract an 8-bit value from the accumulator register, (re)setting the carry,  
    /// half-carry, and zero flags as required.
    pub(super) fn sub(&mut self, n: u8) {
        let value = self.reg(Register::A);
        let hc = half_carry_sub(value, n);
        let (value, carry) = value.overflowing_sub(n);

        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Subtract an 8-bit value and the carry flag from the accumulator register,
    /// (re)setting the carry, half-carry, and zero flags as required.
    pub(super) fn sbc(&mut self, n: u8) {
        let value = self.reg(Register::A);
        let old_carry = self.f.intersects(FlagsRegister::C);

        let hc = half_carry_sbc(value, n, old_carry);
        let (value, carry1) = value.overflowing_sub(n);
        let (value, carry2) = value.overflowing_sub(if old_carry { 1 } else { 0 });

        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry1 || carry2);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Bitwise AND between the accumulator register and an 8-bit value, (re)setting
    /// the zero flag as required.
    pub(super) fn and(&mut self, n: u8) {
        let mut value = self.reg(Register::A);
        value &= n;
        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Bitwise XOR between the accumulator register and an 8-bit value, (re)setting
    /// the zero flag as required.
    pub(super) fn xor(&mut self, n: u8) {
        let mut value = self.reg(Register::A);
        value ^= n;
        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Bitwise OR between the accumulator register and an 8-bit value, (re)setting
    /// the zero flag as required.
    pub(super) fn or(&mut self, n: u8) {
        let mut value = self.reg(Register::A);
        value |= n;
        self.set_reg(Register::A, value);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Subtract an 8-bit value from the accumulator register, (re)setting the carry,  
    /// half-carry, and zero flags as required, but does not store the result.
    pub(super) fn cp(&mut self, n: u8) {
        let value = self.reg(Register::A);
        let hc = half_carry_sub(value, n);
        let (value, carry) = value.overflowing_sub(n);

        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Add two 16-bit registers together, storing the result in the first register
    /// and (re)setting the carry and half-carry flags as required.
    pub(super) fn add_wide(&mut self, lhs: WideRegister, rhs: WideRegister) {
        let x = self.wide_reg(lhs);
        let y = self.wide_reg(rhs);
        let hc = half_carry_add_wide(x, y);
        let (value, carry) = x.overflowing_add(y);

        self.set_wide_reg(lhs, value);
        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);
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

    /// Rotate an 8-bit register left.
    ///
    /// (Re)sets the carry flag.  This operation only changes the zero flag if
    /// `z_flag` is `true`.  
    pub(super) fn rlc(&mut self, reg: Register, z_flag: bool) {
        let mut value = self.reg(reg);
        let carry = value.bit(7);
        value <<= 1;
        if carry {
            value.set_bit(0);
        }
        self.set_reg(reg, value);
        if z_flag {
            self.f.set(FlagsRegister::Z, value == 0);
        }
        self.f.set(FlagsRegister::C, carry);
    }

    /// Rotate an 8-bit register right.
    ///
    /// (Re)sets the carry flag.  This operation only changes the zero flag if
    /// `z_flag` is `true`.  
    pub(super) fn rrc(&mut self, reg: Register, z_flag: bool) {
        let mut value = self.reg(reg);
        let carry = value.bit(0);
        value >>= 1;
        if carry {
            value.set_bit(7);
        }
        self.set_reg(reg, value);
        if z_flag {
            self.f.set(FlagsRegister::Z, value == 0);
        }
        self.f.set(FlagsRegister::C, carry);
    }

    /// Rotate an 8-bit register left through carry.
    ///
    /// (Re)sets the carry flag.  This operation only changes the zero flag if
    /// `z_flag` is `true`.  
    pub(super) fn rl(&mut self, reg: Register, z_flag: bool) {
        let mut value = self.reg(reg);
        let old_carry = self.f.intersects(FlagsRegister::C);
        let carry = value.bit(7);
        value <<= 1;
        if old_carry {
            value.set_bit(0);
        }
        self.set_reg(reg, value);
        if z_flag {
            self.f.set(FlagsRegister::Z, value == 0);
        }
        self.f.set(FlagsRegister::C, carry);
    }

    /// Rotate an 8-bit register right through carry.
    ///
    /// (Re)sets the carry flag.  This operation only changes the zero flag if
    /// `z_flag` is `true`.  
    pub(super) fn rr(&mut self, reg: Register, z_flag: bool) {
        let mut value = self.reg(reg);
        let old_carry = self.f.intersects(FlagsRegister::C);
        let carry = value.bit(0);
        value >>= 1;
        if old_carry {
            value.set_bit(7);
        }
        self.set_reg(reg, value);
        if z_flag {
            self.f.set(FlagsRegister::Z, value == 0);
        }
        self.f.set(FlagsRegister::C, carry);
    }

    /// Rotate an 8-bit register left arithmetically.
    ///
    /// (Re)sets the zero and carry flags as required.
    pub(super) fn sla(&mut self, reg: Register) {
        let mut value = self.reg(reg);
        let carry = value.bit(7);
        value <<= 1;
        self.set_reg(reg, value);
        self.f.set(FlagsRegister::Z, value == 0);
        self.f.set(FlagsRegister::C, carry);
    }

    /// Rotate an 8-bit register right arithmetically.
    ///
    /// (Re)sets the zero and carry flags as required.
    pub(super) fn sra(&mut self, reg: Register) {
        let mut value = self.reg(reg);
        let carry = value.bit(0);
        let bit = value.bit(7);
        value >>= 1;
        if bit {
            value.set_bit(7);
        }
        self.set_reg(reg, value);
        self.f.set(FlagsRegister::Z, value == 0);
        self.f.set(FlagsRegister::C, carry);
    }

    /// Swap the upper nibble in the given register with the lower nibble, (re)setting
    /// the zero flag as required.
    pub(super) fn swap(&mut self, reg: Register) {
        let mut value = self.reg(reg);
        value = ((value & 0x0F) << 4) | (value >> 4);
        self.set_reg(reg, value);
        self.f.set(FlagsRegister::Z, value == 0);
    }

    /// Shift an 8-bit register right logically, (re)setting the zero and carry flags
    /// as required.
    pub(super) fn srl(&mut self, reg: Register) {
        let mut value = self.reg(reg);
        let carry = value.bit(0);
        value >>= 1;
        self.set_reg(reg, value);
        self.f.set(FlagsRegister::Z, value == 0);
        self.f.set(FlagsRegister::C, carry);
    }

    /// Add a signed 8-bit value to the stack pointer, (re)setting the carry and
    /// half carry flags as required.
    pub(super) fn add_sp_offset(&mut self, offset: i8) -> u16 {
        let value = self.sp;
        let bytes = value.to_le_bytes();
        let unsigned_offset = 0u8.wrapping_add_signed(offset);

        let hc = half_carry_add(bytes[0], unsigned_offset);
        let (_, carry) = bytes[0].overflowing_add(unsigned_offset);

        self.f.set(FlagsRegister::H, hc);
        self.f.set(FlagsRegister::C, carry);

        value.wrapping_add_signed(offset.into())
    }
}

/// Check if a half carry will occur when adding `x` and `y`.
fn half_carry_add(x: u8, y: u8) -> bool {
    (((x & 0x0F) + (y & 0x0F)) & 0x10) == 0x10
}

/// Check if a half carry will occur when adding `x` to `y` and the carry flag;
/// here, `c` is the value of the carry flag.
fn half_carry_adc(x: u8, y: u8, c: bool) -> bool {
    let c = if c { 1 } else { 0 };
    (((x & 0x0F) + (y & 0x0F) + c) & 0x10) == 0x10
}

/// Check if a half carry will occur when subtracting `y` from `x`.
fn half_carry_sub(x: u8, y: u8) -> bool {
    ((x & 0x0F).wrapping_sub(y & 0x0F)) & 0x10 == 0x10
}

/// Check if a half carry will occur when subtracting `y` and the carry flag
/// from `x`; here, `c` is the value of the carry flag.
fn half_carry_sbc(x: u8, y: u8, c: bool) -> bool {
    let c = if c { 1 } else { 0 };
    (x & 0x0F) < (y & 0x0F) + c
}

/// Check if a half carry will occur when adding `x` and `y`.
fn half_carry_add_wide(x: u16, y: u16) -> bool {
    (((x & 0x0FFF) + (y & 0x0FFF)) & 0x1000) == 0x1000
}

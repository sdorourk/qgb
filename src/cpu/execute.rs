use std::fmt::Debug;

use crate::{
    bits::Bits,
    mmu::{ReadWriteMemory, Tick},
    TCycles,
};

use super::{
    instruction::Instruction,
    opcode::{Opcode, Register, WideRegister},
    Cpu, FlagsRegister,
};

impl Instruction {
    pub(super) fn execute<T>(&self, cpu: &mut Cpu<T>) -> TCycles
    where
        T: Debug + ReadWriteMemory + Tick,
    {
        match self.opcode {
            Opcode::Nop => self.cycles,
            Opcode::LdDerefImmSp(addr) => {
                cpu.sp = addr;
                self.cycles
            }
            Opcode::Stop => {
                tracing::warn!(target: "cpu", "STOP instruction is not supported");
                self.cycles
            }
            Opcode::Jr(offset) => {
                cpu.pc = cpu.pc.wrapping_sub(self.length);
                cpu.pc = cpu.pc.wrapping_add_signed(i16::from(offset));
                self.cycles
            }
            Opcode::JrCond(cond, offset) => {
                if cpu.condition(cond) {
                    cpu.pc = cpu.pc.wrapping_sub(self.length);
                    cpu.pc = cpu.pc.wrapping_add_signed(i16::from(offset));
                    self.branch_cycles
                } else {
                    self.cycles
                }
            }
            Opcode::LdWideRegImm(reg, value) => {
                cpu.set_wide_reg(reg, value);
                self.cycles
            }
            Opcode::AddHLWideReg(reg) => {
                let sum = cpu.add_wide(WideRegister::HL, reg);
                cpu.set_wide_reg(WideRegister::HL, sum);
                self.cycles
            }
            Opcode::LdDerefWideRegA(reg) => {
                let addr = cpu.wide_reg(reg);
                cpu.write(addr, cpu.a);
                self.cycles
            }
            Opcode::LdADerefWideReg(reg) => {
                let addr = cpu.wide_reg(reg);
                let value = cpu.read(addr);
                cpu.set_reg(Register::A, value);
                self.cycles
            }
            Opcode::LdHLIncA => {
                let value = cpu.reg(Register::A);
                cpu.set_reg(Register::DerefHL, value);
                cpu.inc_wide(WideRegister::HL);
                self.cycles
            }
            Opcode::LdHLDecA => {
                let value = cpu.reg(Register::A);
                cpu.set_reg(Register::DerefHL, value);
                cpu.dec_wide(WideRegister::HL);
                self.cycles
            }
            Opcode::LdAHLInc => {
                let value = cpu.reg(Register::DerefHL);
                cpu.set_reg(Register::A, value);
                cpu.inc_wide(WideRegister::HL);
                self.cycles
            }
            Opcode::LdAHLDec => {
                let value = cpu.reg(Register::DerefHL);
                cpu.set_reg(Register::A, value);
                cpu.dec_wide(WideRegister::HL);
                self.cycles
            }
            Opcode::IncWideReg(reg) => {
                cpu.inc_wide(reg);
                self.cycles
            }
            Opcode::DecWideReg(reg) => {
                cpu.dec_wide(reg);
                self.cycles
            }
            Opcode::IncReg(reg) => {
                cpu.inc(reg);
                self.cycles
            }
            Opcode::DecReg(reg) => {
                cpu.dec(reg);
                self.cycles
            }
            Opcode::LdRegImm(reg, n) => {
                cpu.set_reg(reg, n);
                self.cycles
            }
            Opcode::Rlca => {
                let mut value = cpu.reg(Register::A);
                let carry = value.bit(7);
                value <<= 1;
                if carry {
                    value.set_bit(0);
                }
                cpu.set_reg(Register::A, value);
                cpu.f.set(FlagsRegister::C, carry);
                self.cycles
            }
            // Opcode::Rrca => todo!(),
            // Opcode::Rla => todo!(),
            // Opcode::Rra => todo!(),
            // Opcode::Daa => todo!(),
            // Opcode::Cpl => todo!(),
            // Opcode::Scf => todo!(),
            // Opcode::Ccf => todo!(),
            Opcode::Ld(reg1, reg2) => {
                let value = cpu.reg(reg2);
                cpu.set_reg(reg1, value);
                self.cycles
            }
            // Opcode::Halt => todo!(),
            // Opcode::Add(_) => todo!(),
            // Opcode::Adc(_) => todo!(),
            // Opcode::Sub(_) => todo!(),
            // Opcode::Sbc(_) => todo!(),
            // Opcode::And(_) => todo!(),
            // Opcode::Xor(_) => todo!(),
            // Opcode::Or(_) => todo!(),
            // Opcode::Cp(_) => todo!(),
            // Opcode::RetCond(_) => todo!(),
            // Opcode::LdOffsetImmA(_) => todo!(),
            // Opcode::AddSpDisp(_) => todo!(),
            // Opcode::LdAOffsetImm(_) => todo!(),
            // Opcode::LdHLSPDisp(_) => todo!(),
            // Opcode::PopWideReg(_) => todo!(),
            // Opcode::Ret => todo!(),
            // Opcode::Reti => todo!(),
            // Opcode::JpHL => todo!(),
            // Opcode::LdSPHL => todo!(),
            // Opcode::JPCondImm(_, _) => todo!(),
            // Opcode::LdOffsetCA => todo!(),
            // Opcode::LdDerefImmA(_) => todo!(),
            // Opcode::LdAOffsetC => todo!(),
            // Opcode::LdADerefImm(_) => todo!(),
            // Opcode::JP(_) => todo!(),
            // Opcode::DI => todo!(),
            // Opcode::EI => todo!(),
            // Opcode::CallCondImm(_, _) => todo!(),
            // Opcode::PushWideReg(_) => todo!(),
            // Opcode::CallImm(_) => todo!(),
            // Opcode::AddImm(_) => todo!(),
            // Opcode::AdcImm(_) => todo!(),
            // Opcode::SubImm(_) => todo!(),
            // Opcode::SbcImm(_) => todo!(),
            // Opcode::AndImm(_) => todo!(),
            // Opcode::XorImm(_) => todo!(),
            // Opcode::OrImm(_) => todo!(),
            // Opcode::CpImm(_) => todo!(),
            // Opcode::Rst(_) => todo!(),
            // Opcode::Rlc(_) => todo!(),
            // Opcode::Rrc(_) => todo!(),
            // Opcode::Rl(_) => todo!(),
            // Opcode::Rr(_) => todo!(),
            // Opcode::Sla(_) => todo!(),
            // Opcode::Sra(_) => todo!(),
            // Opcode::Swap(_) => todo!(),
            // Opcode::Srl(_) => todo!(),
            // Opcode::Bit(_, _) => todo!(),
            // Opcode::Res(_, _) => todo!(),
            // Opcode::Set(_, _) => todo!(),
            _ => self.cycles,
        }
    }
}

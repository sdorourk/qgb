use std::fmt::Debug;

use crate::{
    bits::Bits,
    components::mmu::{ReadWriteMemory, Tick},
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
                cpu.add_wide(WideRegister::HL, reg);
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
                cpu.rlc(Register::A, false);
                self.cycles
            }
            Opcode::Rrca => {
                cpu.rrc(Register::A, false);
                self.cycles
            }
            Opcode::Rla => {
                cpu.rl(Register::A, false);
                self.cycles
            }
            Opcode::Rra => {
                cpu.rr(Register::A, false);
                self.cycles
            }
            Opcode::Daa => {
                let mut value = cpu.reg(Register::A);
                if !cpu.f.intersects(FlagsRegister::N) {
                    if cpu.f.intersects(FlagsRegister::C) || value > 0x99 {
                        value = value.wrapping_add(0x60);
                        cpu.f.insert(FlagsRegister::C);
                    }
                    if cpu.f.intersects(FlagsRegister::H) || (value & 0x0F) > 0x09 {
                        value = value.wrapping_add(0x06);
                    }
                } else {
                    if cpu.f.intersects(FlagsRegister::C) {
                        value = value.wrapping_sub(0x60);
                    }
                    if cpu.f.intersects(FlagsRegister::H) {
                        value = value.wrapping_sub(0x06);
                    }
                }
                cpu.set_reg(Register::A, value);
                cpu.f.set(FlagsRegister::Z, value == 0);
                self.cycles
            }
            Opcode::Cpl => {
                let mut value = cpu.reg(Register::A);
                value = !value;
                cpu.set_reg(Register::A, value);
                self.cycles
            }
            Opcode::Scf => self.cycles,
            Opcode::Ccf => {
                cpu.f.toggle(FlagsRegister::C);
                self.cycles
            }
            Opcode::Ld(reg1, reg2) => {
                let value = cpu.reg(reg2);
                cpu.set_reg(reg1, value);
                self.cycles
            }
            // Opcode::Halt => todo!(),
            Opcode::Add(reg) => {
                let value = cpu.reg(reg);
                cpu.add(value);
                self.cycles
            }
            Opcode::Adc(reg) => {
                let value = cpu.reg(reg);
                cpu.adc(value);
                self.cycles
            }
            Opcode::Sub(reg) => {
                let value = cpu.reg(reg);
                cpu.sub(value);
                self.cycles
            }
            Opcode::Sbc(reg) => {
                let value = cpu.reg(reg);
                cpu.sbc(value);
                self.cycles
            }
            Opcode::And(reg) => {
                let value = cpu.reg(reg);
                cpu.and(value);
                self.cycles
            }
            Opcode::Xor(reg) => {
                let value = cpu.reg(reg);
                cpu.xor(value);
                self.cycles
            }
            Opcode::Or(reg) => {
                let value = cpu.reg(reg);
                cpu.or(value);
                self.cycles
            }
            Opcode::Cp(reg) => {
                let value = cpu.reg(reg);
                cpu.cp(value);
                self.cycles
            }
            Opcode::RetCond(cond) => {
                if cpu.condition(cond) {
                    cpu.ret();
                    self.branch_cycles
                } else {
                    self.cycles
                }
            }
            Opcode::LdOffsetImmA(offset) => {
                let value = cpu.reg(Register::A);
                cpu.write(0xFF00 + u16::from(offset), value);
                self.cycles
            }
            Opcode::AddSpDisp(offset) => {
                cpu.sp = cpu.add_sp_offset(offset);
                self.cycles
            }
            Opcode::LdAOffsetImm(offset) => {
                let value = cpu.read(0xFF00 + u16::from(offset));
                cpu.set_reg(Register::A, value);
                self.cycles
            }
            Opcode::LdHLSPDisp(offset) => {
                let value = cpu.add_sp_offset(offset);
                cpu.set_wide_reg(WideRegister::HL, value);
                self.cycles
            }
            Opcode::PopWideReg(reg) => {
                cpu.pop_wide_reg(reg);
                self.cycles
            }
            Opcode::Ret => {
                cpu.ret();
                self.cycles
            }
            // Opcode::Reti => todo!(),
            Opcode::JpHL => {
                let addr = cpu.wide_reg(WideRegister::HL);
                cpu.pc = addr;
                self.cycles
            }
            Opcode::LdSPHL => {
                let addr = cpu.wide_reg(WideRegister::HL);
                cpu.sp = addr;
                self.cycles
            }
            Opcode::JPCondImm(cond, addr) => {
                if cpu.condition(cond) {
                    cpu.pc = addr;
                    self.branch_cycles
                } else {
                    self.cycles
                }
            }
            Opcode::LdOffsetCA => {
                let a = cpu.reg(Register::A);
                let c = cpu.reg(Register::C);
                let addr: u16 = 0xFF00 + u16::from(c);
                cpu.write(addr, a);
                self.cycles
            }
            Opcode::LdDerefImmA(addr) => {
                let value = cpu.reg(Register::A);
                cpu.write(addr, value);
                self.cycles
            }
            Opcode::LdAOffsetC => {
                let c = cpu.reg(Register::C);
                let addr = 0xFF00 + u16::from(c);
                let value = cpu.read(addr);
                cpu.set_reg(Register::A, value);
                self.cycles
            }
            Opcode::LdADerefImm(addr) => {
                let value = cpu.read(addr);
                cpu.set_reg(Register::A, value);
                self.cycles
            }
            Opcode::JP(addr) => {
                cpu.pc = addr;
                self.cycles
            }
            // Opcode::DI => todo!(),
            // Opcode::EI => todo!(),
            Opcode::CallCondImm(cond, addr) => {
                if cpu.condition(cond) {
                    cpu.call(addr);
                    self.branch_cycles
                } else {
                    self.cycles
                }
            }
            Opcode::PushWideReg(reg) => {
                cpu.push_wide_reg(reg);
                self.cycles
            }
            Opcode::CallImm(addr) => {
                cpu.call(addr);
                self.cycles
            }
            Opcode::AddImm(n) => {
                cpu.add(n);
                self.cycles
            }
            Opcode::AdcImm(n) => {
                cpu.adc(n);
                self.cycles
            }
            Opcode::SubImm(n) => {
                cpu.sub(n);
                self.cycles
            }
            Opcode::SbcImm(n) => {
                cpu.sbc(n);
                self.cycles
            }
            Opcode::AndImm(n) => {
                cpu.and(n);
                self.cycles
            }
            Opcode::XorImm(n) => {
                cpu.xor(n);
                self.cycles
            }
            Opcode::OrImm(n) => {
                cpu.or(n);
                self.cycles
            }
            Opcode::CpImm(n) => {
                cpu.cp(n);
                self.cycles
            }
            Opcode::Rst(addr) => {
                cpu.call(addr.into());
                self.cycles
            }
            Opcode::Rlc(reg) => {
                cpu.rlc(reg, true);
                self.cycles
            }
            Opcode::Rrc(reg) => {
                cpu.rrc(reg, true);
                self.cycles
            }
            Opcode::Rl(reg) => {
                cpu.rl(reg, true);
                self.cycles
            }
            Opcode::Rr(reg) => {
                cpu.rr(reg, true);
                self.cycles
            }
            Opcode::Sla(reg) => {
                cpu.sla(reg);
                self.cycles
            }
            Opcode::Sra(reg) => {
                cpu.sra(reg);
                self.cycles
            }
            Opcode::Swap(reg) => {
                cpu.swap(reg);
                self.cycles
            }
            Opcode::Srl(reg) => {
                cpu.srl(reg);
                self.cycles
            }
            Opcode::Bit(bit, reg) => {
                let value = cpu.reg(reg);
                cpu.f.set(FlagsRegister::Z, !value.bit(bit.into()));
                self.cycles
            }
            Opcode::Res(bit, reg) => {
                let mut value = cpu.reg(reg);
                value.reset_bit(bit.into());
                cpu.set_reg(reg, value);
                self.cycles
            }
            Opcode::Set(bit, reg) => {
                let mut value = cpu.reg(reg);
                value.set_bit(bit.into());
                cpu.set_reg(reg, value);
                self.cycles
            }
            _ => self.cycles,
        }
    }
}

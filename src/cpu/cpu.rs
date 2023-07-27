use std::fmt::Debug;

use bitflags::bitflags;

use crate::{
    mmu::{ReadWriteMemory, Tick},
    TCycles,
};

use super::{
    instruction,
    opcode::{Register, WideRegister},
};

/// Number of cycles required to read or write a byte from memory
const DEFAULT_READ_WRITE_CYCLES: TCycles = 4;

/// Number of cycles required to read or write two bytes from memory
const DEFAULT_READ_WRITE_U16_CYCLES: TCycles = DEFAULT_READ_WRITE_CYCLES * 2;

#[derive(Debug)]
pub struct Cpu<T>
where
    T: Debug,
{
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
    /// Memory management unit
    pub mmu: T,
    /// Number of cycles executed by the MMU (and its components) from reading/writing
    /// to memory while executing the current instruction
    pub rw_cycles: TCycles,
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

impl<T> Cpu<T>
where
    T: Debug,
{
    pub fn new(mmu: T) -> Self {
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
            mmu,
            rw_cycles: 0,
        }
    }
}
impl<T> Cpu<T>
where
    T: Debug + ReadWriteMemory + Tick,
{
    /// Execute the next instruction.
    ///
    /// Returns the number of cycles required to execute the instruction.
    pub fn step(&mut self) -> TCycles {
        let instr = match self.fetch() {
            Ok(instr) => instr,
            Err(byte) => {
                tracing::error!(target: "cpu", "unknown opcode encountered (found {:02X}", byte);
                return DEFAULT_READ_WRITE_CYCLES;
            }
        };

        self.execute(instr)
    }

    pub fn fetch(&self) -> Result<instruction::Instruction, u8> {
        let mut stream = MmuByteStream {
            pc: self.pc,
            mmu: &self.mmu,
        };
        instruction::Instruction::try_from(&mut stream)
    }
}

impl<T> Cpu<T>
where
    T: Debug + ReadWriteMemory + Tick,
{
    /// Execute the given `Instruction`.
    ///
    /// Returns the number of cycles required to execute the instruction.
    #[tracing::instrument(target = "cpu", level = "trace", ret)]
    fn execute(&mut self, instr: instruction::Instruction) -> TCycles {
        tracing::trace!(target: "cpu", "executing {}", instr.opcode);

        // Increment the program counter as this is the default behavior for most operations
        self.pc = self.pc.wrapping_add(instr.length);

        // Each component should execute the number of cycles required to read the
        // instructions operands
        self.mmu.tick(instr.read_cycles);
        self.rw_cycles = instr.read_cycles;

        let total_cycles = instr.execute(self);

        self.f.insert(instr.set_flags);
        self.f.remove(instr.reset_flags);

        self.mmu.tick(total_cycles - self.rw_cycles);

        tracing::trace!(target: "cpu", cpu = ?self);
        total_cycles
    }
}

impl<T> Cpu<T>
where
    T: Debug + ReadWriteMemory + Tick,
{
    fn read(&mut self, addr: u16) -> u8 {
        let value = self.mmu.read(addr);
        self.rw_cycles += DEFAULT_READ_WRITE_CYCLES;
        self.mmu.tick(DEFAULT_READ_WRITE_CYCLES);
        value
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.mmu.write(addr, value);
        self.rw_cycles += DEFAULT_READ_WRITE_CYCLES;
        self.mmu.tick(DEFAULT_READ_WRITE_CYCLES);
    }

    fn read_u16(&mut self, addr: u16) -> u16 {
        let value = self.mmu.read_u16(addr);
        self.rw_cycles += DEFAULT_READ_WRITE_U16_CYCLES;
        self.mmu.tick(DEFAULT_READ_WRITE_U16_CYCLES);
        value
    }

    fn write_u16(&mut self, addr: u16, value: u16) {
        self.mmu.write_u16(addr, value);
        self.rw_cycles += DEFAULT_READ_WRITE_U16_CYCLES;
        self.mmu.tick(DEFAULT_READ_WRITE_U16_CYCLES);
    }
}

struct MmuByteStream<'a, T>
where
    T: ReadWriteMemory,
{
    pc: u16,
    mmu: &'a T,
}

impl<'a, T> instruction::ByteStream for MmuByteStream<'a, T>
where
    T: ReadWriteMemory,
{
    fn fetch(&mut self) -> u8 {
        let old_pc = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.mmu.read(old_pc)
    }
}

impl<T> Cpu<T>
where
    T: Debug + ReadWriteMemory + Tick,
{
    fn reg(&mut self, reg: Register) -> u8 {
        match reg {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::DerefHL => self.read(self.wide_reg(WideRegister::HL)),
        }
    }

    fn set_reg(&mut self, reg: Register, value: u8) {
        match reg {
            Register::A => self.a = value,
            Register::B => self.b = value,
            Register::C => self.c = value,
            Register::D => self.d = value,
            Register::E => self.e = value,
            Register::H => self.h = value,
            Register::L => self.l = value,
            Register::DerefHL => self.write(self.wide_reg(WideRegister::HL), value),
        }
    }

    fn wide_reg(&self, reg: WideRegister) -> u16 {
        match reg {
            WideRegister::BC => u16::from_le_bytes([self.c, self.b]),
            WideRegister::DE => u16::from_le_bytes([self.e, self.d]),
            WideRegister::HL => u16::from_le_bytes([self.l, self.h]),
            WideRegister::SP => self.sp,
            WideRegister::AF => u16::from_le_bytes([self.f.bits(), self.a]),
        }
    }

    fn set_wide_reg(&mut self, reg: WideRegister, value: u16) {
        let bytes = value.to_le_bytes();
        match reg {
            WideRegister::BC => {
                self.c = bytes[0];
                self.b = bytes[1];
            }
            WideRegister::DE => {
                self.e = bytes[0];
                self.d = bytes[1];
            }
            WideRegister::HL => {
                self.l = bytes[0];
                self.h = bytes[1];
            }
            WideRegister::SP => self.sp = value,
            WideRegister::AF => {
                self.f = FlagsRegister::from_bits_truncate(bytes[0]);
                self.a = bytes[1];
            }
        }
    }
}

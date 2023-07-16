use std::fmt::Display;

use crate::bits::Bits;

use super::instruction::ByteStream;

/// Various octal digits and bits of a byte
///
/// Used to algorithmically determine which `Opcode` a given byte corresponds to.
/// The fields are based on the following:
///  - x = the bytes's 1st octal digit (i.e., bits 7-6)
///  - y = the bytes's 2nd octal digit (i.e., bits 5-3)
///  - z = the bytes's 3rd octal digit (i.e., bits 2-0)
///  - p = y right shifted one position (i.e., bits 5-4)
///  - q = y modulo 2 (i.e., bit 3)
#[derive(Debug)]
struct DecomposedByte {
    pub byte: u8,
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub p: u8,
    pub q: u8,
}

impl DecomposedByte {
    fn new(byte: u8) -> Self {
        Self {
            byte,
            x: byte.bits(6..=7),
            y: byte.bits(3..=5),
            z: byte.bits(0..=2),
            p: byte.bits(4..=5),
            q: byte.bits(3..=3),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Opcode {
    Nop,
    LdDerefImmSp(u16),
    Stop,
    Jr(i8),
    JrCond(FlagCondition, i8),
    LdWideRegImm(WideRegister, u16),
    AddHLWideReg(WideRegister),
    LdDerefWideRegA(WideRegister),
    LdADerefWideReg(WideRegister),
    LdHLIncA,
    LdHLDecA,
    LdAHLInc,
    LdAHLDec,
    IncWideReg(WideRegister),
    DecWideReg(WideRegister),
    IncReg(Register),
    DecReg(Register),
    LdRegImm(Register, u8),
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    Ld(Register, Register),
    Halt,
    Add(Register),
    Adc(Register),
    Sub(Register),
    Sbc(Register),
    And(Register),
    Xor(Register),
    Or(Register),
    Cp(Register),
    RetCond(FlagCondition),
    LdOffsetImmA(u8),
    AddSpDisp(i8),
    LdAOffsetImm(u8),
    LdHLSPDisp(i8),
    PopWideReg(WideRegister),
    Ret,
    Reti,
    JpHL,
    LdSPHL,
    JPCondImm(FlagCondition, u16),
    LdOffsetCA,
    LdDerefImmA(u16),
    LdAOffsetC,
    LdADerefImm(u16),
    JP(u16),
    DI,
    EI,
    CallCondImm(FlagCondition, u16),
    PushWideReg(WideRegister),
    CallImm(u16),
    AddImm(u8),
    AdcImm(u8),
    SubImm(u8),
    SbcImm(u8),
    AndImm(u8),
    XorImm(u8),
    OrImm(u8),
    CpImm(u8),
    Rst(u8),
    Rlc(Register),
    Rrc(Register),
    Rl(Register),
    Rr(Register),
    Sla(Register),
    Sra(Register),
    Swap(Register),
    Srl(Register),
    Bit(u8, Register),
    Res(u8, Register),
    Set(u8, Register),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::Nop => write!(f, "NOP"),
            Opcode::LdDerefImmSp(nn) => write!(f, "LD (${:04X}), SP", nn),
            Opcode::Stop => write!(f, "STOP"),
            Opcode::Jr(d) => write!(f, "JR {}", d),
            Opcode::JrCond(cond, d) => write!(f, "JR {}, {}", cond, d),
            Opcode::LdWideRegImm(wreg, nn) => write!(f, "LD {}, ${:04X}", wreg, nn),
            Opcode::AddHLWideReg(wreg) => write!(f, "ADD HL, {}", wreg),
            Opcode::LdDerefWideRegA(wreg) => write!(f, "LD ({}), A", wreg),
            Opcode::LdADerefWideReg(wreg) => write!(f, "LD A, ({})", wreg),
            Opcode::LdHLIncA => write!(f, "LD (HL+), A"),
            Opcode::LdHLDecA => write!(f, "LD (HL-), A"),
            Opcode::LdAHLInc => write!(f, "LD A, (HL+)"),
            Opcode::LdAHLDec => write!(f, "LD A, (HL-)"),
            Opcode::IncWideReg(wreg) => write!(f, "INC {}", wreg),
            Opcode::DecWideReg(wreg) => write!(f, "DEC {}", wreg),
            Opcode::IncReg(reg) => write!(f, "INC {}", reg),
            Opcode::DecReg(reg) => write!(f, "DEC {}", reg),
            Opcode::LdRegImm(reg, n) => write!(f, "LD {}, ${:02X}", reg, n),
            Opcode::Rlca => write!(f, "RLCA"),
            Opcode::Rrca => write!(f, "RRCA"),
            Opcode::Rla => write!(f, "RLA"),
            Opcode::Rra => write!(f, "RRA"),
            Opcode::Daa => write!(f, "DAA"),
            Opcode::Cpl => write!(f, "CPL"),
            Opcode::Scf => write!(f, "SCF"),
            Opcode::Ccf => write!(f, "CCF"),
            Opcode::Ld(reg1, reg2) => write!(f, "LD {}, {}", reg1, reg2),
            Opcode::Halt => write!(f, "HALT"),
            Opcode::Add(reg) => write!(f, "ADD A, {}", reg),
            Opcode::Adc(reg) => write!(f, "ADC A, {}", reg),
            Opcode::Sub(reg) => write!(f, "SUB A, {}", reg),
            Opcode::Sbc(reg) => write!(f, "SBC A, {}", reg),
            Opcode::And(reg) => write!(f, "AND A, {}", reg),
            Opcode::Xor(reg) => write!(f, "XOR A, {}", reg),
            Opcode::Or(reg) => write!(f, "OR A, {}", reg),
            Opcode::Cp(reg) => write!(f, "CP A, {}", reg),
            Opcode::RetCond(cond) => write!(f, "RET {}", cond),
            Opcode::LdOffsetImmA(n) => write!(f, "LD ($FF00 + ${:02X}), A", n),
            Opcode::AddSpDisp(d) => write!(f, "ADD SP, {}", d),
            Opcode::LdAOffsetImm(n) => write!(f, "LD A, ($FF00 + ${:02X})", n),
            Opcode::LdHLSPDisp(d) => write!(f, "LD HL, SP + {}", d),
            Opcode::PopWideReg(wreg) => write!(f, "POP {}", wreg),
            Opcode::Ret => write!(f, "RET"),
            Opcode::Reti => write!(f, "RETI"),
            Opcode::JpHL => write!(f, "JP HL"),
            Opcode::LdSPHL => write!(f, "LD SP, HL"),
            Opcode::JPCondImm(cond, nn) => write!(f, "JP {}, ${:04X}", cond, nn),
            Opcode::LdOffsetCA => write!(f, "LD ($FF00 + C), A"),
            Opcode::LdDerefImmA(nn) => write!(f, "LD ${:04X}, A", nn),
            Opcode::LdAOffsetC => write!(f, "LD A, ($FF00 + C)"),
            Opcode::LdADerefImm(nn) => write!(f, "LD A, (${:04X})", nn),
            Opcode::JP(nn) => write!(f, "JP ${:04X}", nn),
            Opcode::DI => write!(f, "DI"),
            Opcode::EI => write!(f, "EI"),
            Opcode::CallCondImm(cond, nn) => write!(f, "CALL {}, ${:04X}", cond, nn),
            Opcode::PushWideReg(wreg) => write!(f, "PUSH {}", wreg),
            Opcode::CallImm(nn) => write!(f, "CALL ${:04X}", nn),
            Opcode::AddImm(n) => write!(f, "ADD A, ${:02X}", n),
            Opcode::AdcImm(n) => write!(f, "ADC A, ${:02X}", n),
            Opcode::SubImm(n) => write!(f, "SUB A, ${:02X}", n),
            Opcode::SbcImm(n) => write!(f, "SBC A, ${:02X}", n),
            Opcode::AndImm(n) => write!(f, "AND A, ${:02X}", n),
            Opcode::XorImm(n) => write!(f, "XOR A, ${:02X}", n),
            Opcode::OrImm(n) => write!(f, "OR A, ${:02X}", n),
            Opcode::CpImm(n) => write!(f, "CP A, ${:02X}", n),
            Opcode::Rst(y) => write!(f, "RST ${:02X}", y * 8),
            Opcode::Rlc(reg) => write!(f, "RLC {}", reg),
            Opcode::Rrc(reg) => write!(f, "RRC {}", reg),
            Opcode::Rl(reg) => write!(f, "RL {}", reg),
            Opcode::Rr(reg) => write!(f, "RR {}", reg),
            Opcode::Sla(reg) => write!(f, "SLA {}", reg),
            Opcode::Sra(reg) => write!(f, "SRA {}", reg),
            Opcode::Swap(reg) => write!(f, "SWAP {}", reg),
            Opcode::Srl(reg) => write!(f, "SRL {}", reg),
            Opcode::Bit(y, reg) => write!(f, "BIT {}, {}", y, reg),
            Opcode::Res(y, reg) => write!(f, "RES {}, {}", y, reg),
            Opcode::Set(y, reg) => write!(f, "SET {}, {}", y, reg),
        }
    }
}

impl Opcode {
    /// Decode the next instruction from the given `ByteStream`
    ///
    /// Returns `Err` containing the fetched byte from `ByteStream` if this byte
    /// does not correspond to any opcode.
    pub fn try_from<T>(stream: &mut T) -> Result<Self, u8>
    where
        T: ByteStream,
    {
        let db = DecomposedByte::new(stream.fetch());
        if db.byte == 0xCB {
            return Ok(Opcode::prefix_try_from(stream));
        }
        match (db.x, db.z) {
            (0, 0) => match db.y {
                0 => Ok(Opcode::Nop),
                1 => Ok(Opcode::LdDerefImmSp(stream.fetch_u16())),
                2 => Ok(Opcode::Stop),
                3 => Ok(Opcode::Jr(stream.fetch_i8())),
                y if y < 8 => Ok(Opcode::JrCond(
                    FlagCondition::from(y - 4),
                    stream.fetch_i8(),
                )),
                _ => unreachable!(),
            },
            (0, 1) => match db.q {
                0 => Ok(Opcode::LdWideRegImm(
                    WideRegister::from_rp(db.p),
                    stream.fetch_u16(),
                )),
                1 => Ok(Opcode::AddHLWideReg(WideRegister::from_rp(db.p))),
                _ => unreachable!(),
            },
            (0, 2) => match (db.q, db.p) {
                (0, 0) => Ok(Opcode::LdDerefWideRegA(WideRegister::BC)),
                (0, 1) => Ok(Opcode::LdDerefWideRegA(WideRegister::DE)),
                (0, 2) => Ok(Opcode::LdHLIncA),
                (0, 3) => Ok(Opcode::LdHLDecA),
                (1, 0) => Ok(Opcode::LdADerefWideReg(WideRegister::BC)),
                (1, 1) => Ok(Opcode::LdADerefWideReg(WideRegister::DE)),
                (1, 2) => Ok(Opcode::LdAHLInc),
                (1, 3) => Ok(Opcode::LdAHLDec),
                _ => unreachable!(),
            },
            (0, 3) => match db.q {
                0 => Ok(Opcode::IncWideReg(WideRegister::from_rp(db.p))),
                1 => Ok(Opcode::DecWideReg(WideRegister::from_rp(db.p))),
                _ => unreachable!(),
            },
            (0, 4) => Ok(Opcode::IncReg(Register::from(db.y))),
            (0, 5) => Ok(Opcode::DecReg(Register::from(db.y))),
            (0, 6) => Ok(Opcode::LdRegImm(Register::from(db.y), stream.fetch())),
            (0, 7) => match db.y {
                0 => Ok(Opcode::Rlca),
                1 => Ok(Opcode::Rrca),
                2 => Ok(Opcode::Rla),
                3 => Ok(Opcode::Rra),
                4 => Ok(Opcode::Daa),
                5 => Ok(Opcode::Cpl),
                6 => Ok(Opcode::Scf),
                7 => Ok(Opcode::Ccf),
                _ => unreachable!(),
            },
            (1, z) => {
                if z == 6 && db.y == 6 {
                    Ok(Opcode::Halt)
                } else {
                    Ok(Opcode::Ld(Register::from(db.y), Register::from(z)))
                }
            }
            (2, z) => match db.y {
                0 => Ok(Opcode::Add(Register::from(z))),
                1 => Ok(Opcode::Adc(Register::from(z))),
                2 => Ok(Opcode::Sub(Register::from(z))),
                3 => Ok(Opcode::Sbc(Register::from(z))),
                4 => Ok(Opcode::And(Register::from(z))),
                5 => Ok(Opcode::Xor(Register::from(z))),
                6 => Ok(Opcode::Or(Register::from(z))),
                7 => Ok(Opcode::Cp(Register::from(z))),
                _ => unreachable!(),
            },
            (3, 0) => match db.y {
                0..=3 => Ok(Opcode::RetCond(FlagCondition::from(db.y))),
                4 => Ok(Opcode::LdOffsetImmA(stream.fetch())),
                5 => Ok(Opcode::AddSpDisp(stream.fetch_i8())),
                6 => Ok(Opcode::LdAOffsetImm(stream.fetch())),
                7 => Ok(Opcode::LdHLSPDisp(stream.fetch_i8())),
                _ => unreachable!(),
            },
            (3, 1) => match (db.q, db.p) {
                (0, p) => Ok(Opcode::PopWideReg(WideRegister::from_rp2(p))),
                (1, 0) => Ok(Opcode::Ret),
                (1, 1) => Ok(Opcode::Reti),
                (1, 2) => Ok(Opcode::JpHL),
                (1, 3) => Ok(Opcode::LdSPHL),
                _ => unreachable!(),
            },
            (3, 2) => match db.y {
                0..=3 => Ok(Opcode::JPCondImm(
                    FlagCondition::from(db.y),
                    stream.fetch_u16(),
                )),
                4 => Ok(Opcode::LdOffsetCA),
                5 => Ok(Opcode::LdDerefImmA(stream.fetch_u16())),
                6 => Ok(Opcode::LdAOffsetC),
                7 => Ok(Opcode::LdADerefImm(stream.fetch_u16())),
                _ => unreachable!(),
            },
            (3, 3) => match db.y {
                0 => Ok(Opcode::JP(stream.fetch_u16())),
                2..=5 => Err(db.byte),
                6 => Ok(Opcode::DI),
                7 => Ok(Opcode::EI),
                _ => unreachable!(),
            },
            (3, 4) => match db.y {
                0..=3 => Ok(Opcode::CallCondImm(
                    FlagCondition::from(db.y),
                    stream.fetch_u16(),
                )),
                4..=7 => Err(db.byte),
                _ => unreachable!(),
            },
            (3, 5) => match (db.q, db.p) {
                (0, p) => Ok(Opcode::PushWideReg(WideRegister::from_rp2(p))),
                (1, 0) => Ok(Opcode::CallImm(stream.fetch_u16())),
                (1, 1..=3) => Err(db.byte),
                _ => unreachable!(),
            },
            (3, 6) => match db.y {
                0 => Ok(Opcode::AddImm(stream.fetch())),
                1 => Ok(Opcode::AdcImm(stream.fetch())),
                2 => Ok(Opcode::SubImm(stream.fetch())),
                3 => Ok(Opcode::SbcImm(stream.fetch())),
                4 => Ok(Opcode::AndImm(stream.fetch())),
                5 => Ok(Opcode::XorImm(stream.fetch())),
                6 => Ok(Opcode::OrImm(stream.fetch())),
                7 => Ok(Opcode::CpImm(stream.fetch())),
                _ => unreachable!(),
            },
            (3, 7) => Ok(Opcode::Rst(db.y * 8)),
            _ => unreachable!(),
        }
    }

    fn prefix_try_from<T>(stream: &mut T) -> Opcode
    where
        T: ByteStream,
    {
        let db = DecomposedByte::new(stream.fetch());
        match db.x {
            0 => match db.y {
                0 => Opcode::Rlc(Register::from(db.z)),
                1 => Opcode::Rrc(Register::from(db.z)),
                2 => Opcode::Rl(Register::from(db.z)),
                3 => Opcode::Rr(Register::from(db.z)),
                4 => Opcode::Sla(Register::from(db.z)),
                5 => Opcode::Sra(Register::from(db.z)),
                6 => Opcode::Swap(Register::from(db.z)),
                7 => Opcode::Srl(Register::from(db.z)),
                _ => unreachable!(),
            },
            1 => Opcode::Bit(db.y, Register::from(db.z)),
            2 => Opcode::Res(db.y, Register::from(db.z)),
            3 => Opcode::Set(db.y, Register::from(db.z)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    DerefHL,
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0 => Register::B,
            1 => Register::C,
            2 => Register::D,
            3 => Register::E,
            4 => Register::H,
            5 => Register::L,
            6 => Register::DerefHL,
            7 => Register::A,
            _ => unreachable!(),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::A => write!(f, "A"),
            Register::B => write!(f, "B"),
            Register::C => write!(f, "C"),
            Register::D => write!(f, "D"),
            Register::E => write!(f, "E"),
            Register::H => write!(f, "H"),
            Register::L => write!(f, "L"),
            Register::DerefHL => write!(f, "(HL)"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum WideRegister {
    BC,
    DE,
    HL,
    SP,
    AF,
}

impl WideRegister {
    pub fn from_rp(value: u8) -> Self {
        match value {
            0 => WideRegister::BC,
            1 => WideRegister::DE,
            2 => WideRegister::HL,
            3 => WideRegister::SP,
            _ => unreachable!(),
        }
    }

    pub fn from_rp2(value: u8) -> Self {
        match value {
            0 => WideRegister::BC,
            1 => WideRegister::DE,
            2 => WideRegister::HL,
            3 => WideRegister::AF,
            _ => unreachable!(),
        }
    }
}

impl Display for WideRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WideRegister::BC => write!(f, "BC"),
            WideRegister::DE => write!(f, "DE"),
            WideRegister::HL => write!(f, "HL"),
            WideRegister::SP => write!(f, "SP"),
            WideRegister::AF => write!(f, "AF"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum FlagCondition {
    NZ,
    Z,
    NC,
    C,
}

impl From<u8> for FlagCondition {
    fn from(value: u8) -> Self {
        match value {
            0 => FlagCondition::NZ,
            1 => FlagCondition::Z,
            2 => FlagCondition::NC,
            3 => FlagCondition::C,
            _ => unreachable!(),
        }
    }
}

impl Display for FlagCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagCondition::NZ => write!(f, "NZ"),
            FlagCondition::Z => write!(f, "Z"),
            FlagCondition::NC => write!(f, "NC"),
            FlagCondition::C => write!(f, "C"),
        }
    }
}

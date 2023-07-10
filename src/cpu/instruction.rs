use crate::cpu::opcode::Opcode;

use super::{opcode::Register, FlagsRegister, TCycles};

#[derive(Debug)]
pub struct Instruction {
    /// Instruction opcode
    pub opcode: Opcode,
    /// Length of the instruction in bytes
    pub length: u16,
    /// Number of T-cycles required to read operands from memory
    pub read_cycles: TCycles,
    /// Number of T-cycles without branch
    pub cycles: TCycles,
    /// Number of T-cycles with branch
    pub branch_cycles: TCycles,
    /// CPU flags that are always set by the instruction
    pub set_flags: FlagsRegister,
    /// CPU flags that are always reset by the instruction
    pub reset_flags: FlagsRegister,
}

struct InstructionBuilder {
    opcode: Opcode,
    length: Option<u16>,
    read_cycles: Option<TCycles>,
    cycles: Option<TCycles>,
    branch_cycles: Option<TCycles>,
    set_flags: Option<FlagsRegister>,
    reset_flags: Option<FlagsRegister>,
}

impl InstructionBuilder {
    fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            length: None,
            read_cycles: None,
            cycles: None,
            branch_cycles: None,
            set_flags: None,
            reset_flags: None,
        }
    }

    fn length(&mut self, length: u16) -> &mut Self {
        self.length = Some(length);
        self
    }

    fn read_cycles(&mut self, read_cycles: TCycles) -> &mut Self {
        self.read_cycles = Some(read_cycles);
        self
    }

    fn cycles(&mut self, cycles: TCycles) -> &mut Self {
        self.cycles = Some(cycles);
        self
    }

    fn branch_cycles(&mut self, branch_cycles: TCycles) -> &mut Self {
        self.branch_cycles = Some(branch_cycles);
        self
    }

    fn set_flag(&mut self, flags: FlagsRegister) -> &mut Self {
        match self.set_flags {
            None => self.set_flags = Some(flags),
            Some(set_flags) => self.set_flags = Some(set_flags | flags),
        }
        self
    }

    fn reset_flag(&mut self, flags: FlagsRegister) -> &mut Self {
        match self.reset_flags {
            None => self.reset_flags = Some(flags),
            Some(reset_flags) => self.reset_flags = Some(reset_flags | flags),
        }
        self
    }

    fn build(&mut self) -> Instruction {
        let cycles = self.cycles.unwrap_or(4);
        Instruction {
            opcode: self.opcode,
            length: self.length.unwrap_or(1),
            read_cycles: self.read_cycles.unwrap_or(0),
            cycles,
            branch_cycles: self.branch_cycles.unwrap_or(cycles),
            set_flags: self.set_flags.unwrap_or_default(),
            reset_flags: self.reset_flags.unwrap_or_default(),
        }
    }
}

pub trait ByteStream {
    /// Fetch the next byte
    fn fetch(&mut self) -> u8;

    fn fetch_i8(&mut self) -> i8 {
        self.fetch() as i8
    }

    fn fetch_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.fetch(), self.fetch()])
    }
}

impl Instruction {
    /// Decode the next instruction from the given `ByteStream`
    ///
    /// Returns a `Result` indicating an `Instruction` was successfully decoded or the
    /// next byte does not correspond to an opcode.
    pub fn try_from<T>(stream: &mut T) -> Result<Instruction, u8>
    where
        T: ByteStream,
    {
        let mut instr = InstructionBuilder::new(Opcode::try_from(stream)?);
        match instr.opcode {
            Opcode::Nop => {}
            Opcode::LdDerefImmSp(_) => {
                instr.length(3).read_cycles(8).cycles(20);
            }
            Opcode::Stop => {
                instr.length(2);
            }
            Opcode::Jr(_) => {
                instr.length(2).read_cycles(4).cycles(12);
            }
            Opcode::JrCond(_, _) => {
                instr.length(2).read_cycles(4).cycles(8).branch_cycles(12);
            }
            Opcode::LdWideRegImm(_, _) => {
                instr.length(3).read_cycles(8).cycles(12);
            }
            Opcode::AddHLWideReg(_) => {
                instr.cycles(8).reset_flag(FlagsRegister::N);
            }
            Opcode::LdDerefWideRegA(_) => {
                instr.cycles(8);
            }
            Opcode::LdADerefWideReg(_) => {
                instr.cycles(8);
            }
            Opcode::LdHLIncA => {
                instr.cycles(8);
            }
            Opcode::LdHLDecA => {
                instr.cycles(8);
            }
            Opcode::LdAHLInc => {
                instr.cycles(8);
            }
            Opcode::LdAHLDec => {
                instr.cycles(8);
            }
            Opcode::IncWideReg(_) => {
                instr.cycles(8);
            }
            Opcode::DecWideReg(_) => {
                instr.cycles(8);
            }
            Opcode::IncReg(Register::DerefHL) => {
                instr.cycles(12).reset_flag(FlagsRegister::N);
            }
            Opcode::IncReg(_) => {
                instr.reset_flag(FlagsRegister::N);
            }
            Opcode::DecReg(Register::DerefHL) => {
                instr.cycles(12).set_flag(FlagsRegister::N);
            }
            Opcode::DecReg(_) => {
                instr.set_flag(FlagsRegister::N);
            }
            Opcode::LdRegImm(Register::DerefHL, _) => {
                instr.length(2).read_cycles(4).cycles(12);
            }
            Opcode::LdRegImm(_, _) => {
                instr.length(2).read_cycles(4).cycles(8);
            }
            Opcode::Rlca | Opcode::Rrca | Opcode::Rla | Opcode::Rra => {
                instr.reset_flag(FlagsRegister::Z | FlagsRegister::N | FlagsRegister::H);
            }
            Opcode::Daa => {
                instr.reset_flag(FlagsRegister::H);
            }
            Opcode::Cpl => {
                instr.set_flag(FlagsRegister::N | FlagsRegister::H);
            }
            Opcode::Scf => {
                instr
                    .reset_flag(FlagsRegister::N | FlagsRegister::H)
                    .set_flag(FlagsRegister::C);
            }
            Opcode::Ccf => {
                instr.reset_flag(FlagsRegister::N | FlagsRegister::H);
            }
            Opcode::Ld(Register::DerefHL, _) | Opcode::Ld(_, Register::DerefHL) => {
                instr.cycles(8);
            }
            Opcode::Ld(_, _) => {}
            Opcode::Halt => {}
            Opcode::Add(Register::DerefHL) => {
                instr.cycles(8).reset_flag(FlagsRegister::N);
            }
            Opcode::Add(_) => {
                instr.reset_flag(FlagsRegister::N);
            }
            Opcode::Adc(Register::DerefHL) => {
                instr.cycles(8).reset_flag(FlagsRegister::N);
            }
            Opcode::Adc(_) => {
                instr.reset_flag(FlagsRegister::N);
            }
            Opcode::Sub(Register::DerefHL) => {
                instr.cycles(8).set_flag(FlagsRegister::N);
            }
            Opcode::Sub(_) => {
                instr.set_flag(FlagsRegister::N);
            }
            Opcode::Sbc(Register::DerefHL) => {
                instr.cycles(8).set_flag(FlagsRegister::N);
            }
            Opcode::Sbc(_) => {
                instr.set_flag(FlagsRegister::N);
            }
            Opcode::And(Register::DerefHL) => {
                instr
                    .cycles(8)
                    .set_flag(FlagsRegister::H)
                    .reset_flag(FlagsRegister::N | FlagsRegister::C);
            }
            Opcode::And(_) => {
                instr
                    .set_flag(FlagsRegister::H)
                    .reset_flag(FlagsRegister::N | FlagsRegister::C);
            }
            Opcode::Xor(Register::DerefHL) => {
                instr
                    .cycles(8)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Xor(_) => {
                instr.reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Or(Register::DerefHL) => {
                instr
                    .cycles(8)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Or(_) => {
                instr.reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Cp(Register::DerefHL) => {
                instr.cycles(8).set_flag(FlagsRegister::N);
            }
            Opcode::Cp(_) => {
                instr.set_flag(FlagsRegister::N);
            }
            Opcode::RetCond(_) => {
                instr.cycles(8).branch_cycles(20);
            }
            Opcode::LdOffsetImmA(_) => {
                instr.length(2).read_cycles(4).cycles(12);
            }
            Opcode::AddSpDisp(_) => {
                instr
                    .length(2)
                    .read_cycles(4)
                    .cycles(16)
                    .reset_flag(FlagsRegister::Z | FlagsRegister::N);
            }
            Opcode::LdAOffsetImm(_) => {
                instr.length(2).read_cycles(4).cycles(12);
            }
            Opcode::LdHLSPDisp(_) => {
                instr
                    .length(2)
                    .read_cycles(4)
                    .cycles(12)
                    .reset_flag(FlagsRegister::Z | FlagsRegister::N);
            }
            Opcode::PopWideReg(_) => {
                instr.cycles(12);
            }
            Opcode::Ret | Opcode::Reti => {
                instr.cycles(16);
            }
            Opcode::JpHL => {}
            Opcode::LdSPHL => {
                instr.cycles(8);
            }
            Opcode::JPCondImm(_, _) => {
                instr.length(3).read_cycles(8).cycles(12).branch_cycles(16);
            }
            Opcode::LdOffsetCA => {
                instr.cycles(8);
            }
            Opcode::LdDerefImmA(_) => {
                instr.length(3).read_cycles(8).cycles(16);
            }
            Opcode::LdAOffsetC => {
                instr.cycles(8);
            }
            Opcode::LdADerefImm(_) => {
                instr.length(3).read_cycles(8).cycles(16);
            }
            Opcode::JP(_) => {
                instr.length(3).read_cycles(8).cycles(16);
            }
            Opcode::DI | Opcode::EI => {}
            Opcode::CallCondImm(_, _) => {
                instr.length(3).read_cycles(8).cycles(12).branch_cycles(24);
            }
            Opcode::PushWideReg(_) => {
                instr.cycles(16);
            }
            Opcode::CallImm(_) => {
                instr.length(3).read_cycles(8).cycles(24);
            }
            Opcode::AddImm(_) | Opcode::AdcImm(_) => {
                instr
                    .length(2)
                    .read_cycles(4)
                    .cycles(8)
                    .reset_flag(FlagsRegister::N);
            }
            Opcode::SubImm(_) | Opcode::SbcImm(_) | Opcode::CpImm(_) => {
                instr
                    .length(2)
                    .read_cycles(4)
                    .cycles(8)
                    .set_flag(FlagsRegister::N);
            }
            Opcode::AndImm(_) => {
                instr
                    .length(2)
                    .read_cycles(4)
                    .cycles(8)
                    .reset_flag(FlagsRegister::N | FlagsRegister::C)
                    .set_flag(FlagsRegister::H);
            }
            Opcode::XorImm(_) | Opcode::OrImm(_) => {
                instr
                    .length(2)
                    .read_cycles(4)
                    .cycles(8)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Rst(_) => {
                instr.cycles(16);
            }
            Opcode::Rlc(Register::DerefHL)
            | Opcode::Rrc(Register::DerefHL)
            | Opcode::Rl(Register::DerefHL)
            | Opcode::Rr(Register::DerefHL)
            | Opcode::Sla(Register::DerefHL)
            | Opcode::Sra(Register::DerefHL)
            | Opcode::Srl(Register::DerefHL) => {
                instr
                    .length(2)
                    .cycles(16)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H);
            }
            Opcode::Rlc(_)
            | Opcode::Rrc(_)
            | Opcode::Rl(_)
            | Opcode::Rr(_)
            | Opcode::Sla(_)
            | Opcode::Sra(_)
            | Opcode::Srl(_) => {
                instr
                    .length(2)
                    .cycles(8)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H);
            }
            Opcode::Swap(Register::DerefHL) => {
                instr
                    .length(2)
                    .cycles(16)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Swap(_) => {
                instr
                    .length(2)
                    .cycles(8)
                    .reset_flag(FlagsRegister::N | FlagsRegister::H | FlagsRegister::C);
            }
            Opcode::Bit(_, Register::DerefHL) => {
                instr
                    .length(2)
                    .cycles(12)
                    .reset_flag(FlagsRegister::N)
                    .set_flag(FlagsRegister::H);
            }
            Opcode::Bit(_, _) => {
                instr
                    .length(2)
                    .cycles(8)
                    .reset_flag(FlagsRegister::N)
                    .set_flag(FlagsRegister::H);
            }
            Opcode::Res(_, Register::DerefHL) | Opcode::Set(_, Register::DerefHL) => {
                instr.length(2).cycles(16);
            }
            Opcode::Res(_, _) | Opcode::Set(_, _) => {
                instr.length(2).cycles(8);
            }
        }

        Ok(instr.build())
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::opcode::FlagCondition;

    use super::*;

    #[derive(Debug)]
    enum InstructionParseError {
        /// A byte was read which does not correspond to any opcode
        UnknownInstruction(u8),
        /// Not enough bytes can be fetched from the stream to decode the opcode
        MissingOperand,
    }

    #[derive(Debug)]
    struct VecByteStream<'a> {
        index: usize,
        rom: &'a Vec<u8>,
        bounds_exceeded: bool,
    }

    impl<'a> VecByteStream<'a> {
        fn new(rom: &'a Vec<u8>) -> Self {
            Self {
                index: 0,
                rom,
                bounds_exceeded: false,
            }
        }
    }

    impl<'a> ByteStream for VecByteStream<'a> {
        fn fetch(&mut self) -> u8 {
            if self.index < self.rom.len() {
                self.index += 1;
                self.rom[self.index - 1]
            } else {
                self.bounds_exceeded = true;
                0
            }
        }
    }

    impl<'a> Iterator for VecByteStream<'a> {
        type Item = Result<Instruction, InstructionParseError>;

        fn next(&mut self) -> Option<Self::Item> {
            if !self.bounds_exceeded && self.index < self.rom.len() {
                let instr_result = Instruction::try_from(self);
                if self.bounds_exceeded {
                    Some(Err(InstructionParseError::MissingOperand))
                } else {
                    match instr_result {
                        Ok(instr) => Some(Ok(instr)),
                        Err(byte) => Some(Err(InstructionParseError::UnknownInstruction(byte))),
                    }
                }
            } else {
                None
            }
        }
    }

    #[test]
    pub fn decode_instructions() {
        // TODO: Add remaining opcodes
        let rom = vec![
            0x00, // NOP
            0x08, // LD $FF10, SP
            0x10,
            0xFF,
            0x10, // STOP
            0x18, // JR -22
            -22i8 as u8,
            0x20, // JR NZ, -10
            -10i8 as u8,
        ];
        let correct_opcodes = vec![
            Opcode::Nop,
            Opcode::LdDerefImmSp(0xFF10),
            Opcode::Stop,
            Opcode::Jr(-22),
            Opcode::JrCond(FlagCondition::NZ, -10),
        ];

        let opcodes: Vec<Opcode> = VecByteStream::new(&rom)
            .map(|x| x.unwrap().opcode)
            .collect();
        assert_eq!(opcodes, correct_opcodes);
    }
}

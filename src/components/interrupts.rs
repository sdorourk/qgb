use bitflags::bitflags;

use super::mmu::{InterruptManager, INTERRUPT_ENABLE_REG, INTERRUPT_FLAG};

/// For managing the IE (0xFFFF) and IF (0xFF0F) registers
pub struct InterruptRegisters {
    /// Interrupt enable register (0xFFFF)
    reg_ie: InterruptFlag,
    /// Interrupt flag (0xFF0F)
    reg_if: InterruptFlag,
}

impl InterruptRegisters {
    pub fn new() -> Self {
        Self {
            reg_ie: Default::default(),
            reg_if: Default::default(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            INTERRUPT_ENABLE_REG => self.reg_ie.bits(),
            INTERRUPT_FLAG => self.reg_if.bits(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            INTERRUPT_ENABLE_REG => self.reg_ie = InterruptFlag::from_bits_truncate(value),
            INTERRUPT_FLAG => self.reg_if = InterruptFlag::from_bits_truncate(value),
            _ => unreachable!(),
        }
    }
}

impl InterruptManager for InterruptRegisters {
    fn if_set(&mut self, interrupt: Interrupt) {
        self.reg_if.insert(interrupt.as_interrupt_flag());
    }

    fn if_reset(&mut self, interrupt: Interrupt) {
        self.reg_if.remove(interrupt.as_interrupt_flag());
    }

    fn priority_interrupt(&mut self) -> Option<Interrupt> {
        let flag = self.reg_ie & self.reg_if;
        if flag.contains(InterruptFlag::VBlank) {
            Some(Interrupt::VBlank)
        } else if flag.contains(InterruptFlag::LcdStat) {
            Some(Interrupt::LcdStat)
        } else if flag.contains(InterruptFlag::Timer) {
            Some(Interrupt::Timer)
        } else if flag.contains(InterruptFlag::Serial) {
            Some(Interrupt::Serial)
        } else if flag.contains(InterruptFlag::Joypad) {
            Some(Interrupt::Joypad)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
    pub fn handler_address(&self) -> u16 {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::LcdStat => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }

    pub fn as_interrupt_flag(&self) -> InterruptFlag {
        match self {
            Interrupt::VBlank => InterruptFlag::VBlank,
            Interrupt::LcdStat => InterruptFlag::LcdStat,
            Interrupt::Timer => InterruptFlag::Timer,
            Interrupt::Serial => InterruptFlag::Serial,
            Interrupt::Joypad => InterruptFlag::Joypad,
        }
    }
}

bitflags! {
    /// Flags for the IE and IF registers
    #[derive(Debug, Default, Clone, Copy)]
    pub struct InterruptFlag: u8 {
        const VBlank = 0b0000_0001;
        const LcdStat = 0b0000_0010;
        const Timer = 0b0000_0100;
        const Serial = 0b0000_1000;
        const Joypad = 0b0001_0000;
    }
}

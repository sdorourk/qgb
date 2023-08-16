use crate::{bits::Bits, TCycles};

use super::mmu::{InterruptManager, DIV_REG, TAC_REG, TIMA_REG, TMA_REG};

#[derive(Debug)]
pub struct Timers {
    /// System clock; incremented each T-cycle
    system_clock: u16,
    /// Timer counter register
    tima: u8,
    /// Timer modulo register
    tma: u8,
    /// Timer control register
    tac: u8,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            system_clock: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            DIV_REG => self.system_clock.to_le_bytes()[1],
            TIMA_REG => self.tima,
            TMA_REG => self.tma,
            TAC_REG => self.tac,
            _ => unreachable!(),
        }
    }

    pub fn write<T>(&mut self, addr: u16, value: u8, interrupt_manager: &mut T)
    where
        T: InterruptManager,
    {
        match addr {
            DIV_REG => {
                let prev_system_clock = self.system_clock;
                self.system_clock = 0;
                if self.timer_enabled() && self.falling_edge_detector(prev_system_clock) {
                    self.increment_tima(interrupt_manager);
                }
            }
            TIMA_REG => self.tima = value,
            TMA_REG => self.tma = value,
            TAC_REG => self.tac = value.bits(0..=2),
            _ => unreachable!(),
        }
    }

    pub fn tick<T>(&mut self, cycles: TCycles, interrupt_manager: &mut T)
    where
        T: InterruptManager,
    {
        for _ in 0..cycles {
            self.increment_system_clock(interrupt_manager);
        }
    }

    fn timer_enabled(&self) -> bool {
        self.tac.bit(2)
    }

    fn increment_system_clock<T>(&mut self, interrupt_manager: &mut T)
    where
        T: InterruptManager,
    {
        let prev_system_clock = self.system_clock;
        self.system_clock = self.system_clock.wrapping_add(1);
        if self.timer_enabled() && self.falling_edge_detector(prev_system_clock) {
            self.increment_tima(interrupt_manager);
        }
    }

    fn increment_tima<T>(&mut self, interrupt_manager: &mut T)
    where
        T: InterruptManager,
    {
        let (new_value, overflow) = self.tima.overflowing_add(1);
        self.tima = new_value;
        if overflow {
            interrupt_manager.if_set(super::interrupts::Interrupt::Timer);
            self.tima = self.tma;
        }
    }

    fn falling_edge_detector(&mut self, prev_system_clock: u16) -> bool {
        let prev_bytes = prev_system_clock.to_le_bytes();
        let bytes = self.system_clock.to_le_bytes();
        let bits = match self.tac.bits(0..=1) {
            0 => (prev_bytes[1].bit(1), bytes[1].bit(1)),
            1 => (prev_bytes[0].bit(3), bytes[0].bit(3)),
            2 => (prev_bytes[0].bit(5), bytes[0].bit(5)),
            3 => (prev_bytes[0].bit(7), bytes[1].bit(7)),
            _ => unreachable!(),
        };

        bits.0 && !bits.1
    }
}

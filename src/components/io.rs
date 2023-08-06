//! Joypad and serial transfer input/output handler
//!
//! Todo: Implement interrupts
use crate::{bits::Bits, state::PollState, TCycles};

/// Joypad and serial transfer handler
#[derive(Debug)]
pub struct IoHandler {
    /// 0xFF00 - P1/JOYP - Joypad regiseter
    ///
    /// This value only contains bits 4 and 5 of the register. The other bits are
    /// programmatically computed each time a read request is received.
    joy: u8,
    /// 0xFF01 - SB - serial transfer data
    sb: u8,
    /// 0xFF02 - SC - serial transfer control
    sc: u8,
    /// Bytes sent via serial transfer
    sent_bytes: Vec<u8>,
    /// Remaining cycle count
    remaining_cycles: TCycles,
    /// Joypad to track which keys have been pressed
    joypad: Joypad,
}

#[derive(Debug)]
pub enum JoypadButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug)]
struct Joypad {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    start: bool,
    select: bool,
}

impl Joypad {
    fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            start: false,
            select: false,
        }
    }

    fn button_pressed(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Up => self.up = true,
            JoypadButton::Down => self.down = true,
            JoypadButton::Left => self.left = true,
            JoypadButton::Right => self.right = true,
            JoypadButton::A => self.a = true,
            JoypadButton::B => self.b = true,
            JoypadButton::Start => self.start = true,
            JoypadButton::Select => self.select = true,
        }
    }

    fn button_released(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Up => self.up = false,
            JoypadButton::Down => self.down = false,
            JoypadButton::Left => self.left = false,
            JoypadButton::Right => self.right = false,
            JoypadButton::A => self.a = false,
            JoypadButton::B => self.b = false,
            JoypadButton::Start => self.start = false,
            JoypadButton::Select => self.select = false,
        }
    }
}

impl IoHandler {
    pub fn new() -> Self {
        Self {
            joy: 0,
            sb: 0,
            sc: 0,
            sent_bytes: Vec::new(),
            remaining_cycles: 0,
            joypad: Joypad::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF00 => self.compute_joy(),
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF00 => self.joy = value & 0b0011_0000,
            0xFF01 => self.sb = value,
            0xFF02 => self.sc = value & 0b1000_0001,
            _ => unreachable!(),
        };
    }

    fn compute_joy(&self) -> u8 {
        let mut value = self.joy;
        if !self.joy.bit(5) {
            // Action buttons
            set_bit_condition(&mut value, 3, self.joypad.start);
            set_bit_condition(&mut value, 2, self.joypad.select);
            set_bit_condition(&mut value, 1, self.joypad.b);
            set_bit_condition(&mut value, 0, self.joypad.a);
        } else if !self.joy.bit(4) {
            // Direction buttons
            set_bit_condition(&mut value, 3, self.joypad.down);
            set_bit_condition(&mut value, 2, self.joypad.up);
            set_bit_condition(&mut value, 1, self.joypad.left);
            set_bit_condition(&mut value, 0, self.joypad.right);
        }
        value
    }

    pub fn tick(&mut self, cycles: TCycles) {
        if self.sc == 0x81 {
            self.remaining_cycles += cycles;
            if self.remaining_cycles >= 4 {
                self.remaining_cycles = 0;
                self.serial_transfer_byte();
            } else {
                self.remaining_cycles = 0;
            }
        }
    }

    fn serial_transfer_byte(&mut self) {
        self.sent_bytes.push(self.sb);
        self.sb = 0;
        self.sc.reset_bit(7);
    }

    pub fn button_pressed(&mut self, button: JoypadButton) {
        self.joypad.button_pressed(button);
    }

    pub fn button_released(&mut self, button: JoypadButton) {
        self.joypad.button_released(button);
    }
}

/// Reset bit `index` of `value` if `cond` is `true`; sets the bit if `cond`
/// is `false`.
fn set_bit_condition(value: &mut u8, index: usize, cond: bool) {
    if cond {
        value.reset_bit(index);
    } else {
        value.set_bit(index);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serial_transfer() {
        let mut io = IoHandler::new();
        io.write(0xFF01, 0xAB);
        assert_eq!(io.sb, 0xAB);
        io.write(0xFF02, 0x81);
        io.tick(4);

        assert_eq!(io.sent_bytes.len(), 1);
        assert_eq!(io.sent_bytes[0], 0xAB);
        assert_eq!(io.sc, 0x01);
        assert_eq!(io.sb, 0);
    }

    #[test]
    fn joypad() {
        let mut io = IoHandler::new();
        io.button_pressed(JoypadButton::A);
        io.button_pressed(JoypadButton::Down);

        io.write(0xFF00, 0b0001_0000);
        assert_eq!(io.read(0xFF00), 0b0001_1110);

        io.write(0xFF00, 0b0010_0000);
        assert_eq!(io.read(0xFF00), 0b0010_0111);
    }
}

impl PollState for IoHandler {
    fn poll_state(&self, state: &mut crate::State) {
        if state.io.is_none() {
            state.io = Some(Default::default());
        }
        if let Some(io_state) = &mut state.io {
            io_state.transmitted_bytes = self.sent_bytes.clone();
            io_state.registers.insert("P1".into(), self.compute_joy());
            io_state.registers.insert("SB".into(), self.sb);
            io_state.registers.insert("SC".into(), self.sc);
        }
    }
}

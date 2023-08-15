use crate::{components::interrupts::Interrupt, TCycles};

/// The `ReadWriteMemory` trait allows for reading and writing bytes to memory.
pub trait ReadWriteMemory {
    /// Read a byte from the given address
    fn read(&self, addr: u16) -> u8;

    /// Write `value` to the given memory address
    fn write(&mut self, addr: u16, value: u8);

    /// Read two bytes from the given address
    fn read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr.wrapping_add(1))])
    }

    /// Write `value` to the given memory address
    fn write_u16(&mut self, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.write(addr, bytes[0]);
        self.write(addr.wrapping_add(1), bytes[1]);
    }
}

/// The `Tick` trait is used to synchronizes cycle timing in the system.
pub trait Tick {
    /// Notify the system components that the CPU has executed the given number
    /// of cycles.
    fn tick(&mut self, cycles: TCycles);
}

/// The `InterruptManager` trait is used to manage the IE and IF interrupt registers
pub trait InterruptManager {
    /// Set the flag corresponding to `interrupt` for the IF register
    fn if_set(&mut self, interrupt: Interrupt);

    /// Reset the flag corresponding to `interrupt` for the IF register
    fn if_reset(&mut self, interrupt: Interrupt);

    /// Return the interrupt that should be handled next.
    ///
    /// If more than one bit in the IF register is set, than the returned interrupt is
    /// the one with higher priority (VBlank has the highest priority and Joypad has
    /// the lowest priority).  If no interrupt requires handling, `None` is returned.
    fn priority_interrupt(&mut self) -> Option<Interrupt>;
}

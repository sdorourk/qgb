use crate::TCycles;

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

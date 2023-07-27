//! A dummy implementation of the MMU for testing and debugging purposes.
//!
//! This implementation stores a large memory buffer that it reads and writes to.  It does
//! not implement any components or memory mapped registers.

use super::*;

const MEMORY_SIZE: usize = u16::MAX as usize;

pub struct DummyMmu {
    memory: Box<[u8]>,
}

impl DummyMmu {
    pub fn new(rom: &[u8]) -> Self {
        assert!(rom.len() < MEMORY_SIZE);

        let mut mem = vec![0; MEMORY_SIZE];
        mem[..rom.len()].copy_from_slice(rom);

        Self {
            memory: mem.into_boxed_slice(),
        }
    }
}

impl ReadWriteMemory for DummyMmu {
    fn read(&self, addr: u16) -> u8 {
        self.memory[usize::from(addr)]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.memory[usize::from(addr)] = value;
    }
}

impl Tick for DummyMmu {
    fn tick(&mut self, _cycles: crate::TCycles) {}
}

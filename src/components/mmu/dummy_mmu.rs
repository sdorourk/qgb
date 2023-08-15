//! A dummy implementation of the MMU for testing and debugging purposes.
//!
//! This implementation stores a large memory buffer that it reads and writes to.  It does
//! not implement any components or memory mapped registers.
#![cfg(test)]
use std::fmt::Debug;

use super::*;

const MEMORY_SIZE: usize = 64 * 1024;

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

impl Debug for DummyMmu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DummyMmu").finish_non_exhaustive()
    }
}

// DummyMmu does not handle any interrupts
impl InterruptManager for DummyMmu {
    fn if_set(&mut self, _interrupt: crate::components::interrupts::Interrupt) {}

    fn if_reset(&mut self, _interrupt: crate::components::interrupts::Interrupt) {}

    fn priority_interrupt(&mut self) -> Option<crate::components::interrupts::Interrupt> {
        None
    }
}

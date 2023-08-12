pub mod error;
pub mod state;

use crate::{components::mmu, cpu::cpu, BootError, State};

use self::state::PollState;

pub type TCycles = i64;

pub struct GameBoy {
    pub(crate) cpu: cpu::Cpu<mmu::Mmu>,
    pub state: State,
}

impl GameBoy {
    pub fn new(rom: &[u8], boot_rom: &[u8]) -> Result<Self, BootError> {
        let mmu = mmu::Mmu::new(rom, boot_rom)?;

        Ok(Self {
            cpu: cpu::Cpu::new(mmu),
            state: Default::default(),
        })
    }

    pub fn state(&mut self) -> &State {
        self.cpu.poll_state(&mut self.state);
        &self.state
    }

    pub fn set_instruction_buffer_size(&mut self, buffer_size: usize) {
        self.state.instruction_buffer_size = buffer_size;
    }

    pub fn step(&mut self) -> TCycles {
        self.cpu.step()
    }

    pub fn pc(&self) -> u16 {
        self.cpu.pc
    }
}

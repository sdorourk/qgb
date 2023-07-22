mod bits;
pub mod cpu;

pub struct GameBoy {
    pub(crate) cpu: cpu::Cpu,
}

impl GameBoy {
    pub fn new(_rom: &[u8], _boot_rom: &[u8]) -> Self {
        Self {
            cpu: cpu::Cpu::new(),
        }
    }

    pub fn cpu(&self) -> &cpu::Cpu {
        &self.cpu
    }
}

#[allow(clippy::module_inception)]
pub mod cpu;
mod execute;
pub mod instruction;
pub mod opcode;

pub use cpu::*;

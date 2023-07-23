use std::{fs, path::PathBuf};

use clap::Parser;

use qgb::mmu::ReadWriteMemory;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// ROM program to run
    program: PathBuf,
    /// Boot ROM
    #[arg(short, long)]
    boot_rom: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    init_logger();

    let rom = fs::read(&cli.program).unwrap();
    let boot_rom = fs::read(&cli.boot_rom).unwrap();

    let gb = match qgb::GameBoy::new(&rom, &boot_rom) {
        Ok(gb) => gb,
        Err(qgb::BootError::BootRomError(e)) => {
            eprintln!("'{}': {}", cli.boot_rom.display(), e);
            return;
        }
        Err(qgb::BootError::RomError(e)) => {
            eprintln!("'{}': {}", cli.boot_rom.display(), e);
            return;
        }
    };

    for addr in 0x0000..=0x0010 {
        println!("{:04X}: {:02X}", addr, gb.cpu().mmu.read(addr));
    }
}

fn init_logger() {
    if let Err(err) = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .without_time()
                .with_span_events(fmt::format::FmtSpan::ACTIVE)
                .pretty(),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .try_init()
    {
        eprintln!(
            "an error occurred initializing the tracing library: {}",
            err
        );
    }
}

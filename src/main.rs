mod debugger;

use std::{fs, path::PathBuf, sync::mpsc::channel};

use clap::Parser;

use debugger::Message;
use qgb::TCycles;
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

    run(gb);
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

const CYCLES_PER_SEC: TCycles = 4 * 1024 * 1024;
const FRAMES_PER_SEC: TCycles = 60;
const CYCLES_PER_FRAME: TCycles = CYCLES_PER_SEC / FRAMES_PER_SEC;

#[derive(Debug)]
enum EmulatorRunState {
    Pause,
    Run,
    Step,
}

fn run(mut gb: qgb::GameBoy) {
    let (msg_sender, msg_receiver) = channel::<Message>();
    let mut debugger = debugger::Debugger::new(msg_sender, gb.state());
    debugger.update(gb.state());
    let mut run_state = EmulatorRunState::Pause;
    let mut cycle_count: TCycles = 0;
    loop {
        debugger.handle_events();
        match msg_receiver.try_recv() {
            Ok(Message::Pause) => run_state = EmulatorRunState::Pause,
            Ok(Message::Run) => run_state = EmulatorRunState::Run,
            Ok(Message::Step) => run_state = EmulatorRunState::Step,
            Ok(Message::Quit) => break,
            Err(_) => {}
        }

        match run_state {
            EmulatorRunState::Pause => {
                cycle_count = 0;
            }
            EmulatorRunState::Run => {
                cycle_count += CYCLES_PER_FRAME;
                while cycle_count > 0 {
                    cycle_count -= gb.step();
                }
                debugger.update(gb.state());
            }
            EmulatorRunState::Step => {
                cycle_count = 0;
                gb.step();
                run_state = EmulatorRunState::Pause;
                debugger.update(gb.state());
            }
        }
        std::thread::sleep(std::time::Duration::from_secs_f64(0.016));
    }
}

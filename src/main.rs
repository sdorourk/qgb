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
    /// Console logger (for comparing logs with other emulators)
    #[arg(short, long)]
    console_log: bool,
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

    run(gb, cli.console_log);
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

fn run(mut gb: qgb::GameBoy, console_log: bool) {
    let (msg_sender, msg_receiver) = channel::<Message>();
    let mut debugger = debugger::Debugger::new(msg_sender, gb.state());
    let mut console_logger = DefaultConsoleLogger::new(console_log, false);
    debugger.update(gb.state());
    console_logger.print_log(&mut gb);
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
                let breakpoints = debugger.breakpoints();
                cycle_count += CYCLES_PER_FRAME;
                while cycle_count > 0 {
                    cycle_count -= gb.step();
                    console_logger.print_log(&mut gb);
                    if breakpoints.contains(&gb.pc()) {
                        run_state = EmulatorRunState::Pause;
                        break;
                    }
                }
                debugger.update(gb.state());
            }
            EmulatorRunState::Step => {
                cycle_count = 0;
                gb.step();
                console_logger.print_log(&mut gb);
                run_state = EmulatorRunState::Pause;
                debugger.update(gb.state());
            }
        }
        std::thread::sleep(std::time::Duration::from_secs_f64(0.016));
    }
}

trait ConsoleLogger {
    fn print_log(&mut self, gb: &mut qgb::GameBoy);
}

#[derive(Debug)]
struct DefaultConsoleLogger {
    enabled: bool,
    log_boot_rom: bool,
    boot_rom_ended: bool,
}

impl DefaultConsoleLogger {
    pub fn new(enabled: bool, log_boot_rom: bool) -> Self {
        Self {
            enabled,
            log_boot_rom,
            boot_rom_ended: false,
        }
    }
}

impl ConsoleLogger for DefaultConsoleLogger {
    fn print_log(&mut self, gb: &mut qgb::GameBoy) {
        if self.enabled {
            let state = gb.state();
            if let Some(cpu_state) = &state.cpu {
                if cpu_state.pc == 0x101 {
                    self.boot_rom_ended = true;
                }
                if self.log_boot_rom || self.boot_rom_ended {
                    let mut memory = Vec::new();
                    for i in 0..4 {
                        memory.extend_from_slice(&cpu_state.instructions[i].bytes[..]);
                    }
                    println!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})", cpu_state.a, cpu_state.f, cpu_state.b, cpu_state.c, cpu_state.d, cpu_state.e, cpu_state.h, cpu_state.l, cpu_state.sp, cpu_state.pc, memory[0], memory[1], memory[2], memory[3]);
                }
            }
        }
    }
}

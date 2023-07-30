use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use clap::Parser;

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

    let rom = vec![
        0x04, // INC B
        0x2B, // DEC HL
        0x05, // DEC B
        0x05, // DEC B
        0x06, // LD B, $10
        0x10,
    ];

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

    let mut debugger = Debugger::new(gb);
    debugger.start();
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

struct Debugger {
    gb: qgb::GameBoy,
    break_points: HashSet<u16>,
}

#[derive(Debug)]
enum Command {
    Dump,
    Step,
    Run(usize),
    Exit,
}

impl TryFrom<String> for Command {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let cmd: Vec<String> = value
            .trim()
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if cmd.is_empty() {
            return Err("No command specified".into());
        }
        match cmd[0].as_str() {
            "dump" | "d" => Ok(Command::Dump),
            "step" | "s" => Ok(Command::Step),
            "run" | "r" => {
                if cmd.len() == 1 {
                    Ok(Command::Run(0))
                } else if cmd.len() == 2 {
                    if let Ok(step_size) = cmd[1].parse() {
                        Ok(Command::Run(step_size))
                    } else {
                        Err("Unrecognized step size".into())
                    }
                } else {
                    Err("Incorrect number of arguments specified".into())
                }
            }
            "exit" | "quit" | "e" | "q" => Ok(Command::Exit),
            _ => Err("Invalid command".into()),
        }
    }
}

impl Debugger {
    pub fn new(gb: qgb::GameBoy) -> Self {
        Self {
            gb,
            break_points: HashSet::new(),
        }
    }

    pub fn start(&mut self) {
        println!("Initializing debugger");
        loop {
            self.prologue();
            print!("qgb> ");
            io::stdout().flush().unwrap();

            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let cmd = match Command::try_from(buffer) {
                Ok(cmd) => cmd,
                Err(msg) => {
                    println!("{}", msg);
                    continue;
                }
            };

            match cmd {
                Command::Dump => println!("{:#X?}", self.gb.cpu()),
                Command::Step => {
                    self.gb.step();
                }
                Command::Run(mut steps) => {
                    if steps == 0 {
                        steps = usize::MAX;
                    }
                    for step in 0..steps {
                        self.gb.step();
                        if step != steps - 1 {
                            self.prologue();
                        }
                    }
                }
                Command::Exit => return,
            }
        }
    }

    fn prologue(&self) {
        println!(
            "{:04X}: {}",
            self.gb.cpu().pc,
            self.gb.fetch().unwrap().opcode
        );
    }
}
